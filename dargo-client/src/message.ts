export abstract class Message {
    type = ''
    data: any

    serialise() {
        return JSON.stringify({
            t: this.type,
            d: this.data,
        })
    }
}

type DimensionsData = {
    width: Number;
    height: Number;
    resolution: Number;
}

export class DimensionsMessage extends Message {
    type = 'd'
    data: DimensionsData

    constructor(data: DimensionsData) {
        super()
        this.data = data
    }
}

export class PointerEndEvent extends Message {
    type = 'te'
    data: Number[]

    constructor(ev: PointerEvent | TouchEvent) {
        super()
        if (ev instanceof PointerEvent) {
            this.data = [ev.pointerId]
        } else {
            this.data = Array.from(ev.changedTouches).map(t => t.identifier)
        }
    }
}

export class PointerUpdateEvent extends Message {
    type = 'tu'
    data: {
        id: Number,
        x: Number,
        y: Number,
        rx: Number, // radius_x
        ry: Number, // radius_y
        ra: Number, // rotation_angle
        p: Number   // pressure
    }[]

    constructor(ev: PointerEvent | TouchEvent) {
        super()
        if (ev instanceof PointerEvent) {
            this.data = [{
                id: ev.pointerId,
                x: ev.offsetX,
                y: ev.offsetY,
                rx: ev.width,
                ry: ev.height,
                ra: 0,
                p: ev.pressure
            }]
        } else {
            this.data = Array.from(ev.changedTouches).map(t => ({
                id: t.identifier,
                x: t.clientX,
                y: t.clientY,
                rx: t.radiusX,
                ry: t.radiusY,
                ra: t.rotationAngle,
                p: t.force
            }))
        }
    }
}
