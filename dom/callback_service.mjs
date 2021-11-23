export class CallbackService {
    subscribers = {};

    subscribe(to, subscriberContext, subscriber) {
        if(!this.subscribers[to])
            this.subscribers[to] = [];

        this.subscribers[to].push({context: subscriberContext, subscriber: subscriber});
    }

    get_subscriptions() {
        for (const to in this.subscribers) {
            for (const subscriber in this.subscribers[to]) {
                console.log(this.subscribers[to][subscriber]);
            }
        }
    }

    unsubscribe(to, subscriber) {
        if(!this.subscribers[to])
            return;
        const index = this.subscribers[to].indexOf({context: subscriberContext, subscriber: subscriber});
        if(index > -1) {
            this.subscribers[to].splice(index, 1);
        }
    }

    publish(to, data) {
        for (const index in this.subscribers[to]) {
            const subscriber = this.subscribers[to][index];
            subscriber.subscriber.call(subscriber.context, data);
        }
    }
}