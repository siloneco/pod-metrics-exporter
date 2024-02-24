## Before You Apply the Manifest...

You cannot directly apply the `manifest.yml` to your Kubernetes cluster because I haven't published the Docker image (this is because the software is for personal use and managing container images can be a hassle). So, please build it for your own environment, then modify the image section in the `manifest.yml`, and afterward, deploy it.

```
kubectl apply -n <YOUR_NAMESPACE> -f ./manifest.yml
```