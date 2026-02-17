import { defineStore } from 'pinia'

export const useWebsocketStore = defineStore('websocket', {
    state: () => ({ 
        socket: null,
        isConnected: false,
        channels: new Map(),
     }),

     actions: {
        connect(url) {
            return new Promise((resolve) => {
                if (this.socket) {
                    console.warn('WebSocket is already connected.')
                    resolve()
                    return
                }

                this.socket = new WebSocket(url)

                this.socket.onopen = () => {
                    this.isConnected = true
                    console.log('WebSocket connected.')
                    resolve()
                }

                this.socket.onmessage = (event) => {
                    const msg = JSON.parse(event.data)
                    const result = msg.result

                    const key = `${msg.channel}:${msg.ticker}`
                    const listeners = this.channels.get(key)
                    
                    if (!listeners) {
                        return
                    }
                    listeners.forEach(callback => callback(result))
                }

                this.socket.onerror = (error) => {
                    console.error('WebSocket error:', error)
                    resolve()
                    return;
                }

                this.socket.onclose = () => {
                    this.isConnected = false
                    this.socket = null
                    this.channels.clear()
                    console.log('WebSocket disconnected.')
                }
            })
        },

        subscribe(ticker, channel, longExchange, shortExchange, callback) {
            const key = `${channel}:${ticker}`

            if (!this.channels.has(key)) {
                this.channels.set(key, new Set())
                
                this.send({
                    action: 'subscribe',
                    channel,
                    longExchange,
                    shortExchange,
                    ticker,
                })
            }
            this.channels.get(key).add(callback)

            return () => {                
                const set = this.channels.get(key)

                set.delete(callback)

                if (set.size === 0) {
                    this.send({
                        action: 'unsubscribe',
                        channel,
                        ticker,
                    })
                    this.channels.delete(key)
                }
            }
        },

        send(data) {
            if (this.socket && this.isConnected) {
                this.socket.send(JSON.stringify(data))
                console.log('Sent message:', data)
            } else {
                console.warn('WebSocket is not connected. Cannot send message.')
            }
        },

        disconnect() {
            return new Promise((resolve) => {
                if (this.socket && this.isConnected) {
                    this.socket.close()
                    resolve()
                }
            })
        }
    }
})