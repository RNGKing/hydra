(import ../build/hydra :as db)

(defn execute-in-file-test []
  (print "[ IN FILE TEST ] STARTING IN FILE TEST ...")

  (print "...")

  (print "[ IN FILE TEST ] CREATING DATABASE CONNECTION ...")

  (def in-file-db
    (db/open-local-db "test.db"))

  (print "[ IN FILE TEST ] COMPLETE CREATING DATABASE CONNECTION ...")

  (print "...")

  (print "[ IN FILE TEST ] CREATING TABLE ...")

  (db/execute "CREATE TABLE users (ID INTEGER PRIMARY KEY AUTOINCREMENT, EMAIL TEXT);" @[])

  (print "[ IN FILE TEST ] COMPLETE CREATING TABLE ...")

  (print "...")

  (print "[ IN FILE TEST ] INSERTING VALUES INTO TABLE ...")

  (db/execute "INSERT INTO users (EMAIL) VALUES (?1),(?2),(?3),(?4);"
    @["testone@test.com" "testtwo@test.com" "testthree@test.com" "testfour@test.com"])

  (print "[ IN FILE TEST ] COMPLETE INSERTING VALUE INTO TABLE ...")

  (print "...")

  (print "[ IN FILE TEST ] QUERYING VALUES FROM TABLE ...")

  (db/query "SELECT * FROM users;" @[])

  (print "[ IN FILE TEST ] COMPLETE QUERYING VALUES FROM TABLE ...")

  (print "...")

  (print "[ IN FILE TEST ] CLOSING DATABASE CONNECTION ...")

  (db/close-db in-file-db)

  (print "[ IN FILE TEST ] COMPLETE CLOSING DATABASE CONNECTION ...")

  (print "...")
  
  (print "[ IN FILE TEST ] ENDING IN FILE TEST ..."))
