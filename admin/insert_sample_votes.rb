require 'mysql2'
require 'mysql2-cs-bind'

def config
  @config ||= {
    db: {
      host: ENV['ISHOCON2_DB_HOST'] || 'localhost',
      port: ENV['ISHOCON2_DB_PORT'] && ENV['ISHOCON2_DB_PORT'].to_i,
      username: ENV['ISHOCON2_DB_USER'] || 'root',
      password: ENV['ISHOCON2_DB_PASSWORD'],
      database: ENV['ISHOCON2_DB_NAME'] || 'ishocon2'
    }
  }
end

def db
  return Thread.current[:ishocon2_db] if Thread.current[:ishocon2_db]
  client = Mysql2::Client.new(
    host: config[:db][:host],
    port: config[:db][:port],
    username: config[:db][:username],
    password: config[:db][:password],
    database: config[:db][:database],
    reconnect: true
  )
  client.query_options.merge!(symbolize_keys: true)
  Thread.current[:ishocon2] = client
  client
end

def insert_votes
  db.query('DELETE FROM votes')
  db.query('ALTER TABLE votes AUTO_INCREMENT = 1')
  100.times do |i|
    sql = 'INSERT INTO votes (user_id, candidate_id, keyword) VALUES '
    1000.times do |j|
      user_id = rand(25000000) + 1
      rd = rand(6)
      candidate_id = 0
      if rd == 0
        candidate_id = 3
      elsif rd == 1
        candidate_id = 19
      elsif rd == 2 || rd == 3
        candidate_id = rand(10) + 1
      elsif rd == 4
        candidate_id = rand(20) + 1
      else
        candidate_id = rand(20) + 10
      end
      keyword = %w(誠実 親戚 親近感 柔軟な対応 日本酒はもっと値上げしても良いと思います。稀少性に合った値付けになっていないし、価格が上がれば、資本的にも、人的にも、参入が増えて、優勝劣敗がすすむかと。 ちょーーーーうまいよ！！！ エバラの年間売上が500億円で、うち焼き肉のタレが半分ほど。ライトノベル市場はORICONによると文庫が220億円、近年急速に拡大している四六判なども含めると350億円くらいとのこと)[rand(7)]
      sql += "(#{user_id},#{candidate_id},'#{keyword}'),"
    end
    db.query(sql[0..-2])
    p i
  end
end

insert_votes
