[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

# Data as a Service (DaaS) SDK

For software development teams who implement the [Data as a Service (DaaS)](https://github.com/dsietz/daas) pattern and follow the practice of [Privacy by Design (PbD)](https://github.com/dsietz/pbd), this DaaS SDK provides enablers to help you easily and transparently applying best practices. Unlike other solutions, this SDK bridges the microservice based DaaS architecture pattern with Data Privacy strategies to provide a complete tool kit and saves developers time from having to search, derive, or piece together disparate solutions.

---

**Table of Contents**
- [Data as a Service (DaaS) SDK](#data-as-a-service-daas-sdk)
  - [What's New](#whats-new)
  - [Features](#features)
  - [About](#about)
  - [How to Contribute](#how-to-contribute)
  - [License](#license)

## What's New

Here's whats new in 0.0.2:

1. fixed revisioning of DaaS documents for local storage
2. modified DaaS document to manage binary content (e.g.: mp3)

## Features

- local storage of the DaaS document for listener service
- Kafka brokering as an independent thread when processing the sourced data 

## About

The intent of the `daas-sdk` development kit is to enable the implementation of [DaaS pattern](https://github.com/dsietz/daas) by providing the functionality and components for developers to implement best practices in their own software soltuions. 

## How to Contribute

Details on how to contribute can be found in the [CONTRIBUTING](./CONTRIBUTING.md) file.

## License

`daas-sdk` is primarily distributed under the terms of the Apache License (Version 2.0).

See [LICENSE-APACHE "Apache License](./LICENSE-APACHE) for details.