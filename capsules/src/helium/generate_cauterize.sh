#!/bin/sh

set -e

cauterize msg.scm caut/msg.spec
caut-rust-ref --spec caut/msg.spec --output caut/
