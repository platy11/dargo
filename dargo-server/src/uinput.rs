use std::io;
use anyhow::anyhow;
use evdev_rs::{AbsInfo, DeviceWrapper, EnableCodeData, InputEvent, UInputDevice, UninitDevice};
use evdev_rs::enums::{BusType, EventCode, EV_ABS, EV_KEY, EV_SYN, InputProp};

use crate::message::{Message, Touch};

#[allow(non_camel_case_types, dead_code, clippy::enum_variant_names)]
enum MT_TOOL {
    // https://github.com/torvalds/linux/blob/v6.3/include/uapi/linux/input.h#L279
    // Not in evdev_rs :c
    MT_TOOL_FINGER = 0,
    MT_TOOL_PEN = 1,
    MT_TOOL_PALM = 2,
    MT_TOOL_DIAL = 0xa,
    MT_TOOL_MAX = 0xf,
}

#[cfg(target_os = "linux")]
pub struct UinputTrackpad {
    /// Underlying libevdev device
    dev: UInputDevice,

    /// Maps multitouch slot ids to the client's touch tracking ids
    slots: [Option<i32>; Self::NUM_SLOTS],

    /// Queue of active touches and their last recorded position, oldest first.
    /// Used for [UinputTrackpad::report_legacy_and_tool].
    active_touches: Vec<(usize, [i32; 2])>
}

impl UinputTrackpad {
    const NUM_SLOTS: usize = 10;

    // For details on what these numbers (and below in ::new) mean, see
    // https://github.com/torvalds/linux/blob/v6.3/include/uapi/linux/input.h#L66
    const SLOT_INFO: EnableCodeData = EnableCodeData::AbsInfo(AbsInfo{
        value: 0, minimum: 0, maximum: Self::NUM_SLOTS as i32, fuzz: 0, flat: 0, resolution: 0
    });
    const TRACKING_ID_INFO: EnableCodeData = EnableCodeData::AbsInfo(AbsInfo{
        value: 0, minimum: 0, maximum: 15, fuzz: 0, flat: 0, resolution: 0
    });
    const TOOL_TYPE_INFO: EnableCodeData = EnableCodeData::AbsInfo(AbsInfo{
        value: 0, minimum: 0, maximum: MT_TOOL::MT_TOOL_MAX as i32, fuzz: 0, flat: 0, resolution: 0
    });
    const AXIS_INFO: AbsInfo = AbsInfo{
        // maximum/resolution overriden before use
        value: 0, minimum: 0, maximum: 0, fuzz: 1, flat: 0, resolution: 0
    };

    pub fn new(width: i32, height: i32, resolution: i32) -> anyhow::Result<Self> {
        let dev = Self::prepare_uinput_device(width, height, resolution)?;
        Ok(Self{
            dev,
            slots: [None; Self::NUM_SLOTS],
            active_touches: Vec::with_capacity(Self::NUM_SLOTS)
        })
    }

    fn prepare_uinput_device(width: i32, height: i32, resolution: i32) -> anyhow::Result<UInputDevice> {
        let dev = UninitDevice::new().ok_or(anyhow!("couldn't create uninitialised libevdev device"))?;
        dev.set_name("Dargo virtual trackpad");
        dev.set_bustype(BusType::BUS_VIRTUAL as u16);

        // For libinput to care about the events we send, the virtual device
        // needs to have been labelled with ID_INPUT_TOUCHPAD=1 by udev. This
        // requires enabling and/or sending the right events and properties.
        // In particular, we must:
        // - activate the BTN_TOUCH key when there is at least one active touch
        // - activate the relevant BTN_TOOL_* key corresponding to the number of touches
        // - supply the position of all points through ABS_MT_* events
        //   according to the multitouch protocol
        // - supply the position of the oldest active touch through ABS_X and
        //   ABS_Y in case some client doesn't understand multitouch events
        // - set INPUT_PROP_POINTER to indicate that this is a touchpad (not
        //   e.g. a touchscreen, which would use the same event types)
        // For further information, see:
        // - https://kernel.org/doc/html/v4.19/input/multi-touch-protocol.html
        // - https://github.com/systemd/systemd/blob/v253/src/udev/udev-builtin-input_id.c#L316
        // - https://gitlab.freedesktop.org/libinput/libinput/-/blob/1.23.0/src/evdev-mt-touchpad.c#L3501

        dev.enable(EventCode::EV_KEY(EV_KEY::BTN_TOUCH))?;
        dev.enable(EventCode::EV_KEY(EV_KEY::BTN_TOOL_FINGER))?;
        dev.enable(EventCode::EV_KEY(EV_KEY::BTN_TOOL_DOUBLETAP))?;
        dev.enable(EventCode::EV_KEY(EV_KEY::BTN_TOOL_TRIPLETAP))?;
        dev.enable(EventCode::EV_KEY(EV_KEY::BTN_TOOL_QUADTAP))?;
        dev.enable(EventCode::EV_KEY(EV_KEY::BTN_TOOL_QUINTTAP))?;
        dev.enable_property(&InputProp::INPUT_PROP_POINTER)?;

        dev.enable_event_code(&EventCode::EV_ABS(EV_ABS::ABS_MT_SLOT), Some(UinputTrackpad::SLOT_INFO))?;
        dev.enable_event_code(&EventCode::EV_ABS(EV_ABS::ABS_MT_TRACKING_ID), Some(UinputTrackpad::TRACKING_ID_INFO))?;
        dev.enable_event_code(&EventCode::EV_ABS(EV_ABS::ABS_MT_TOOL_TYPE), Some(UinputTrackpad::TOOL_TYPE_INFO))?;

        let x_info = Some(EnableCodeData::AbsInfo(AbsInfo{
            maximum: width, resolution, ..Self::AXIS_INFO
        }));
        let y_info = Some(EnableCodeData::AbsInfo(AbsInfo{
            maximum: height, resolution, ..Self::AXIS_INFO
        }));

        dev.enable_event_code(&EventCode::EV_ABS(EV_ABS::ABS_X), x_info)?;
        dev.enable_event_code(&EventCode::EV_ABS(EV_ABS::ABS_MT_POSITION_X), x_info)?;
        dev.enable_event_code(&EventCode::EV_ABS(EV_ABS::ABS_Y), y_info)?;
        dev.enable_event_code(&EventCode::EV_ABS(EV_ABS::ABS_MT_POSITION_Y), y_info)?;

        Ok(UInputDevice::create_from_device(&dev)?)
    }

    fn update_dimensions(&mut self, width: i32, height: i32, resolution: i32) -> anyhow::Result<()> {
        // Ideally, we would be able to update the absinfo without creating a
        // new underlying libinput device. Currently this can't be done
        // directly since evdev_rs::UInputDevice doesn't implement
        // DeviceWrapper (in particular, .set_abs_info).
        // TODO: investigate whether this is a limitation of libevdev or just
        // the bindings
        self.dev = Self::prepare_uinput_device(width, height, resolution)?;
        self.slots = [None; Self::NUM_SLOTS];
        self.active_touches.clear();
        Ok(())
    }

    fn event(&self, event_code: EventCode, value: i32) -> io::Result<()> {
        self.dev.write_event(&InputEvent{
            event_code,
            value,
            // A meaningful timestamp isn't needed for uinput events
            time: evdev_rs::TimeVal{tv_sec: 0, tv_usec: 0}
        })
    }

    /// Reports events for ABS_X, ABS_Y, BTN_TOUCH and BTN_TOOL_* as required.
    // Analogous to input_mt_report_finger_count and input_mt_report_pointer_emulation
    // in Linux's drivers/input/input-mt.c
    // https://github.com/torvalds/linux/blob/master/drivers/input/input-mt.c#L164-L257
    fn report_legacy_and_tool(&self) -> io::Result<()> {
        let count = self.active_touches.len();

        if count != 0 {
            let (_, [x, y]) = self.active_touches[0];
            self.event(EventCode::EV_ABS(EV_ABS::ABS_X), x)?;
            self.event(EventCode::EV_ABS(EV_ABS::ABS_Y), y)?;
        }

        self.event(EventCode::EV_KEY(EV_KEY::BTN_TOUCH), (count != 0) as i32)?;
        self.event(EventCode::EV_KEY(EV_KEY::BTN_TOOL_FINGER), (count == 1) as i32)?;
        self.event(EventCode::EV_KEY(EV_KEY::BTN_TOOL_DOUBLETAP), (count == 2) as i32)?;
        self.event(EventCode::EV_KEY(EV_KEY::BTN_TOOL_TRIPLETAP), (count == 3) as i32)?;
        self.event(EventCode::EV_KEY(EV_KEY::BTN_TOOL_QUADTAP), (count == 4) as i32)?;
        self.event(EventCode::EV_KEY(EV_KEY::BTN_TOOL_QUINTTAP), (count == 5) as i32)?;
        Ok(())
    }

    fn report_mt_slot(&self, slot_id: i32, touch: Option<Touch>) -> io::Result<()> {
        self.event(EventCode::EV_ABS(EV_ABS::ABS_MT_SLOT), slot_id)?;
        if let Some(touch) = touch {
            self.event(EventCode::EV_ABS(EV_ABS::ABS_MT_TOOL_TYPE), MT_TOOL::MT_TOOL_FINGER as i32)?;
            self.event(EventCode::EV_ABS(EV_ABS::ABS_MT_TRACKING_ID), touch.id)?;
            self.event(EventCode::EV_ABS(EV_ABS::ABS_MT_POSITION_X), touch.x as i32)?;
            self.event(EventCode::EV_ABS(EV_ABS::ABS_MT_POSITION_Y), touch.y as i32)?;
        } else {
            self.event(EventCode::EV_ABS(EV_ABS::ABS_MT_TRACKING_ID), -1)?;
        }
        Ok(())
    }

    /// Returns (slot_id, new)
    fn find_slot_for_id(&self, tracking_id: i32) -> anyhow::Result<(usize, bool)> {
        for (i, id) in self.slots.iter().enumerate() {
            if id.as_ref() == Some(&tracking_id) {
                return Ok((i, false))
            }
        }
        // tracking_id not known, need to assign to an empty slot
        for (i, id) in self.slots.iter().enumerate() {
            if id.is_none() {
                return Ok((i, true))
            }
        }
        Err(anyhow!("cannot assign touch to slot"))
    }

    pub fn process_message(&mut self, msg: Message) -> anyhow::Result<()> {
        match msg {
            Message::DimensionsUpdate(data) => self.update_dimensions(
                data.width, data.height, data.resolution
            ),
            Message::TouchUpdate(touches) => self.process_touch_update(touches),
            Message::TouchEnd(ids) => self.process_touch_end(ids),
        }?;

        self.report_legacy_and_tool()?;
        self.event(EventCode::EV_SYN(EV_SYN::SYN_REPORT), 0)?;
        Ok(())
    }

    fn process_touch_update(&mut self, touches: Vec<Touch>) -> anyhow::Result<()> {
        for touch in touches {
            let (slot_id, new_touch) = self.find_slot_for_id(touch.id)?;
            self.slots[slot_id] = Some(touch.id);
            self.report_mt_slot(slot_id as i32, Some(touch))?;

            let pos = [touch.x as i32, touch.y as i32];
            if new_touch {
                self.active_touches.push((slot_id, pos));
            } else {
                let index = self.active_touches
                    .iter()
                    .position(|(id, _)| *id == slot_id)
                    .unwrap();
                self.active_touches[index] = (slot_id, pos);
            }
        }
        Ok(())
    }

    fn process_touch_end(&mut self, ids: Vec<i32>) -> anyhow::Result<()> {
        for id in ids {
            let (slot_id, _) = self.find_slot_for_id(id)?;
            self.slots[slot_id] = None;
            self.report_mt_slot(slot_id as i32, None)?;

            for (i, (id, _)) in self.active_touches.iter().enumerate() {
                if *id == slot_id {
                    self.active_touches.remove(i);
                    break
                }
            }
        }
        Ok(())
    }
}
