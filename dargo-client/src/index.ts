import { DimensionsMessage, PointerEndEvent, PointerUpdateEvent } from './message'
import { Socket, SocketState } from './socket'

const trackpad = document.querySelector('#trackpad')
const indicator = document.querySelector('#indicator')

if (location.search == '?touch') {
    localStorage.setItem('useTouchEvents', 'yes')
} else if (location.search == '?pointer') {
    localStorage.removeItem('useTouchEvents')
}
const useTouchEvents = localStorage.getItem('useTouchEvents')

class App {
    socket: Socket;

    constructor({ url, maxRetries }) {
        this.socket = new Socket(url)
        this.socket.maxRetries = maxRetries
    }

    sendDimensions() {
        let width = trackpad.clientWidth
        let height = trackpad.clientHeight

        // resolution is supposed to be the number of arbitrary units for
        // x/y per 1mm. this estimate assumes that when devicePixelRatio =
        // 1, 1px = 1/96 in, which should be good enough. (1in = 25.4mm)
        let resolution = Math.round(96 * window.devicePixelRatio / 25.4)
        this.socket.send(new DimensionsMessage({ width, height, resolution }))
        console.log(`sent dimensions: width ${width} height ${height} resolution ${resolution}`)
    }

    updated(ev: PointerEvent | TouchEvent) {
        if (this.socket.state != SocketState.CONNECTED) return
        if (ev instanceof PointerEvent && ev.pressure == 0) return
        ev.preventDefault()
        this.socket.send(new PointerUpdateEvent(ev))
    }

    ended(ev: PointerEvent | TouchEvent) {
        if (this.socket.state != SocketState.CONNECTED) return
        this.socket.send(new PointerEndEvent(ev))
    }
}

const app = new App({
    url: '/api/socket',
    maxRetries: 5
})

app.socket.addEventListener('connstatechange', () => {
    console.log(`ws state change (new: ${app.socket.state})`)
    indicator.setAttribute('class', app.socket.state)
    indicator.setAttribute('title',
        {
            [SocketState.CONNECTED]: 'Connected',
            [SocketState.CONNECTING]: 'Connecting',
            [SocketState.DISCONNECTED]: 'Failed to connect, reload to retry'
        }[app.socket.state]
    )

    if (app.socket.state == SocketState.CONNECTED) {
        app.sendDimensions()
    }
})

window.addEventListener('resize', _ => app.sendDimensions())

if (useTouchEvents) {
    console.log('using touch events')
    trackpad.addEventListener('touchstart', ev => app.updated(ev as TouchEvent))
    trackpad.addEventListener('touchmove', ev => app.updated(ev as TouchEvent))
    trackpad.addEventListener('touchcancel', ev => app.ended(ev as TouchEvent))
    trackpad.addEventListener('touchend', ev => app.ended(ev as TouchEvent))
} else {
    console.log('using pointer events')
    trackpad.addEventListener('pointerdown', ev => app.updated(ev as PointerEvent))
    trackpad.addEventListener('pointermove', ev => app.updated(ev as PointerEvent))
    trackpad.addEventListener('pointercancel', ev => app.ended(ev as PointerEvent))
    trackpad.addEventListener('pointerup', ev => app.ended(ev as PointerEvent))
}
