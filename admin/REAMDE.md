# イメージの自動 build について

Docker Hubの [automated-build](https://docs.docker.com/docker-hub/builds/#create-an-automated-build) 機能を使って、master ブランチに push されると、自動でイメージをビルドして Docker Hub に登録されるようになっている。

`Dockerfile` とイメージの対応は以下の通り

* `Dockerfile_app`: `ishocon2_app:latest`
* `Dockerfile_bench`: `ishocon2_bench:latest`

# 手動でやる場合
## アプリケーションイメージ作成手順(メモ)
```
$ docker build -f Dockerfile_app . -t showwin/ishocon2_app:$version
$ docker login
$ docker push showwin/ishocon2_app:$version
```

## ベンチマーカーイメージ作成手順(メモ)
```
$ docker build -f Dockerfile_bench . -t showwin/ishocon2_bench:$version
$ docker login
$ docker push showwin/ishocon2_bench:$version
```
