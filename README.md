[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Coverage Status](https://coveralls.io/repos/github/dsietz/daas-sdk/badge.svg?branch=master)](https://coveralls.io/github/dsietz/daas-sdk?branch=master)
[![Docs.rs](https://docs.rs/daas/badge.svg)](https://docs.rs/daas)

Linux: [![Build Status](https://travis-ci.org/dsietz/daas-sdk.svg?branch=master)](https://travis-ci.org/dsietz/daas-sdk)
Windows: [![Build status](https://ci.appveyor.com/api/projects/status/ws0gwwlr2hgiqsiv/branch/master?svg=true)](https://ci.appveyor.com/project/dsietz/daas-sdk)

# Data as a Service (DaaS) SDK

For software development teams who implement the [Data as a Service (DaaS)](https://github.com/dsietz/daas) pattern and follow the practice of [Privacy by Design (PbD)](https://github.com/dsietz/pbd), this DaaS SDK provides enablers to help you easily and transparently applying best practices. Unlike other solutions, this SDK bridges the microservice based DaaS architecture pattern with Data Privacy strategies to provide a complete tool kit and saves developers time from having to search, derive, or piece together disparate solutions.

---

**Table of Contents**
- [Data as a Service (DaaS) SDK](#data-as-a-service-daas-sdk)
  - [What's New](#whats-new)
  - [Features](#features)
  - [Examples](#examples)
      - [Starting the DaaS listening Service](#starting-the-daas-listening-service)
      - [Starting the DaaS Genesis Processor](#starting-the-daas-genesis-processor)
      - [Starting the Order Clothing Processor](#starting-the-order-clothing-processor)
      - [Sourcing the Data](#sourcing-the-data)
  - [About](#about)
  - [How to Contribute](#how-to-contribute)
  - [License](#license)

## What's New

Here's whats new in 0.0.4:

1. Data Processor service module

## Features

- Privacy by Design
- local storage of the DaaS document for listener service
- Kafka brokering as an independent thread when processing the sourced data 
- Processor service traits for building custom data processors
- Out of box Geneis processor for managing the raw data and start of all data flows

## Examples 
This SDK comes with examples for each of the key services for the DaaS pattern.

#### Starting the DaaS listening Service
```
C:\workspace\daas-sdk> cargo build --example daas-listener
C:\workspace\daas-sdk> cd .\target\debug\examples\
C:\workspace\daas-sdk\target\debug\examples> .\daas-listener.exe
```

#### Starting the DaaS Genesis Processor
> NOTE: This requires that you have set up a S3 Bucket with the AWS crendentials set as environment variables
```
C:\workspace\daas-sdk> cargo build --example genesis
C:\workspace\daas-sdk> cd .\target\debug\examples\
C:\workspace\daas-sdk\target\debug\examples> .\genesis.exe
```

#### Starting the Order Clothing Processor
```
C:\workspace\daas-sdk> cargo build --example order-clothing
C:\workspace\daas-sdk> cd .\target\debug\examples\
C:\workspace\daas-sdk\target\debug\examples> .\order-clothing.exe
```

#### Sourcing the Data
There is a `daas-sdk` Collection in the `./examples/postman` directory of this repo that contains example RESTful calls that can be imported and run from Postman.

## About

The intent of the `daas-sdk` development kit is to enable the implementation of [DaaS pattern](https://github.com/dsietz/daas) by providing the functionality and components for developers to implement best practices in their own software soltuions. 

## How to Contribute

Details on how to contribute can be found in the [CONTRIBUTING](./CONTRIBUTING.md) file.

## License

`daas-sdk` is primarily distributed under the terms of the Apache License (Version 2.0).

See [LICENSE-APACHE "Apache License](./LICENSE-APACHE) for details.