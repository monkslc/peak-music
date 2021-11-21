# Peak Infrastructure

## Configuring the peak admin role for the aws cli
1. Run `aws configure --profile peak`
2. Paste in the Access Key and the Secret Access key which can be found in LastPass
3. Set the default region to `us-east-2`

## SSH into box
1. Download the private key from LastPass and save it to `~/.ssh/peak-music.pem`
2. Edit file permissions
```
chmod 400 ~/.ssh/peak-music.pem
````
3. Add the following entry to `~/.ssh/config`
```
Host api.peak.band
    User ec2-user
    IdentityFile ~/.ssh/peak-music.pem
```
4. SSH
```
ssh api.peak.band
```

## Deploying new infrastructure changes
`npm run deploy`
