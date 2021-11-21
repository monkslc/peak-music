import { App } from "@aws-cdk/core";

import { MainStack } from "./stacks/main";

const app = new App();
new MainStack(app, "MainStack", { env: {region: "us-east-2" }});
