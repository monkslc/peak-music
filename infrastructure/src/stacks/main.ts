import { Construct, Stack, StackProps } from "@aws-cdk/core";
import {
    AmazonLinuxCpuType,
    AmazonLinuxGeneration,
    CfnEIP,
    CfnEIPAssociation,
    Instance,
    InstanceType,
    MachineImage,
    Peer,
    Port,
    SecurityGroup,
    SubnetType,
    Vpc,
} from "@aws-cdk/aws-ec2";
import { ARecord, HostedZone, RecordTarget } from "@aws-cdk/aws-route53";
import { Role, ServicePrincipal } from "@aws-cdk/aws-iam";

const PEAK_BAND_ZONE_ID = "Z019212522KE2ITU9E7G";

export class MainStack extends Stack {
    constructor(scope: Construct, id: string, props?: StackProps) {
        super(scope, id, props);

        const vpc = new Vpc(this, "vpc", {
            cidr: "10.0.0.0/16",
            natGateways: 0,
        });

        const sg = new SecurityGroup(this, "security-group", {
            vpc,
            securityGroupName: "Peak Music VPC Security Group",
        });
        sg.addIngressRule(
            Peer.anyIpv4(),
            Port.tcp(22),
            "Allow ssh access from anywhere",
        );
        sg.addIngressRule(
            Peer.anyIpv4(),
            Port.tcp(80),
            "Allows HTTP access from anywhere"
        );
        sg.addIngressRule(
            Peer.anyIpv4(),
            Port.tcp(443),
            "Allows HTTPS access from anywhere"
        );

        const backendRunner = new Role(
            this,
            "backend-runner", { assumedBy: new ServicePrincipal("ec2.amazonaws.com") }
        );

        const instance = new Instance(this, "peak-backend", {
            vpc,
            vpcSubnets: { subnetType: SubnetType.PUBLIC },
            role: backendRunner,
            securityGroup: sg,
            instanceName: "Backend Server",
            instanceType: new InstanceType("t4g.nano"),
            machineImage: MachineImage.latestAmazonLinux({
                cpuType: AmazonLinuxCpuType.ARM_64,
                generation: AmazonLinuxGeneration.AMAZON_LINUX_2,
            }),
            keyName: "peak-admin-key",
        });

        const ip = new CfnEIP(this, "peak-backend-ip", {
            domain: "vpc",
            instanceId: instance.instanceId,
        });

        new CfnEIPAssociation(this, "peak-backend-ip-assoc", {
            eip: ip.ref,
            instanceId: instance.instanceId,
        });

        const zone = HostedZone.fromHostedZoneAttributes(this, "peak-music-hosted-zone", {
            zoneName: "peak.band",
            hostedZoneId: PEAK_BAND_ZONE_ID,
        });

        new ARecord(this, "api-a-record", {
            zone,
            recordName: "api",
            target: RecordTarget.fromIpAddresses(ip.ref),
        });
    }
}
