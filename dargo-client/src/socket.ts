import {Message} from "./message"

export class Socket extends EventTarget {
    #state: SocketState = SocketState.CONNECTING
    #retries = 0
    maxRetries = 0
    #ws: WebSocket

    get state(): SocketState {
        return this.#state
    }

    set state(s: SocketState) {
        const change = s != this.#state
        this.#state = s
        if (change) {
            this.dispatchEvent(new Event('connstatechange'))
        }
    }

    constructor(url: string) {
        super()
        this.#createSocket(url)
    }

    #createSocket(url: string) {
        const err = (error: Error) => {
            if (this.#retries < this.maxRetries) {
                this.state = SocketState.CONNECTING
                this.#retries++
                setTimeout(() => {
                    this.#createSocket(url)
                }, (2 ** this.#retries) * 500)
            } else {
                this.state = SocketState.DISCONNECTED
                throw error
            }
        }

        try {
            this.#ws = new WebSocket(url)
            // this.#ws.addEventListener('message', this.#onMessage)
            this.#ws.addEventListener('open', () => {
                this.state = SocketState.CONNECTED
            })
            this.#ws.addEventListener('close', closeEvent => {
                // @ts-ignore: `cause` isn't supported in the es6 target,
                // but it shouldn't be an issue to pass it anyway
                err(new Error('WebSocket closed', {
                    cause: closeEvent
                }))
            })
        } catch (e) {
            err(e)
        }
    }

    async send(message: Message) {
        if (this.state != SocketState.CONNECTED) throw Error('socket not ready')
        this.#ws.send(message.serialise())
    }
}

export enum SocketState {
    CONNECTING = 'connecting',
    CONNECTED = 'connected',
    DISCONNECTED = 'disconnected'
}
