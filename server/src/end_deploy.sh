#!/bin/bash
sed -i.bak "s/const DEPLOY_OR_STATIC: \&str = \"deploy\";/const DEPLOY_OR_STATIC: \&str = \"static\";/g; s&.secure(true)&// .secure(true)&g" html_handlers.rs &&rm html_handlers.rs.bak
