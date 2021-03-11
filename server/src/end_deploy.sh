#!/bin/bash
sed -i.bak "s&.secure(true)&// .secure(true)&g" html_handlers.rs &&rm html_handlers.rs.bak
