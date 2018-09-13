#!/bin/sh

set -e

cauterize msg.scm msg.spec
caut-rust-ref --spec ./msg.spec --output ../../../caut-rust
