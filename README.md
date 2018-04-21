# actix-web-tower-service-tryouts

  <a href="https://travis-ci.org/o0Ignition0o/actix-web-tower-service-tryouts">
      <img src="https://img.shields.io/travis/o0Ignition0o/actix-web-tower-service-tryouts/master.svg" alt="Travis Build Status">
  </a>


This project is using the same Code of Conduct as actix projects do.
It is licensed as Apache2/MIT as well as actix projects.
# License
<p>
  <a href="LICENSE-APACHE">
    <img
    src="https://img.shields.io/badge/license-apache2-green.svg" alt="MPL 2.0 License">
  </a>
  <a href="LICENSE-MIT">
    <img
    src="https://img.shields.io/badge/license-mit-blue.svg" alt="MIT License">
  </a>
</p>


Some really basic tryouts on how tower and actix-web work. Trying to figure out if both would be easily interoperable.

expected outputs : 

Once you started the project, go to your browser, or use curl to make an insecure request to https://127.0.0.1:8443/

```
$ curl -k https://127.0.0.1:8443/                          
Hello world, welcome to the actix tower-service test ! I've been invoked 1 times so far :)

$ curl -k https://127.0.0.1:8443/\?name\=jeremy                              
Hello jeremy, welcome to the actix tower-service test ! I've been invoked 2 times so far :)
```

# Run the example : 
- cargo run

# Test the example :
- cargo test