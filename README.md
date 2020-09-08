## usage

nginx and mysql up
```
$ docker-compose up
# 初回だけ、以下実行する。dumpデータがDBに入る。1度実行すればあとはローカルのファイルシステムに保存される
$ docker/init_db.sh
```

rust app run
```
$ cargo run 8080
```

flamegraphの出し方

docker/start_app.sh のコメントアウト部分を外し、docker-compose build && docker-compose up
