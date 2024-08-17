# Motivation
This is mostly for local development where webhooks cannot reach localhost. Usually people would use something like a Relay server or tunneling to expose local ports to the internet. It works, but there are some networks that you just do not want exposed :).
So, the idea is to use XHR redirects to send the webhooks to localhost. While, this could still work by just running the localhost environment, there are still some pain points that we need to consider, such as CORS. If possible we want to make the dev environment to be
as close as possible to production and have better isolation between processes and networks.

Enter Interceder, while it still sucks, it gets the job done. You can have it hosted inside a container for somewhat better isolation, or heck add a reverse proxy then connect it to a Relay server. Just make sure
that it is always trusted locally. Interceder will then sit between both networks and delegate the webhooks to where it should actually go.

# TODO
- [ ] tests
- [ ] usage
- [ ] license
- [ ] ghcr
