export class CallbackService extends Object {
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

    unsubscribe(to, context) {
        if(!this.subscribers[to])
            return;
        
        for (const sub of this.subscribers[to]) {
            if(sub.context == context) {
                const index = this.subscribers[to].indexOf(sub);
                if(index > -1) {
                    this.subscribers[to].splice(index, 1);
                }
                return;
            }
        }
    }

    publish(to, data) {
        for (const index in this.subscribers[to]) {
            const subscriber = this.subscribers[to][index];
            subscriber.subscriber.call(subscriber.context, data);
        }
    }
}