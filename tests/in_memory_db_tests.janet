(import ../build/hydra :as db)

(defn execute-in-memory-test []
  (print "[ IN MEMORY TEST ] STARTING IN MEMORY TESTING ...")

  (print "...")

  (def in-memory-db
    (db/open-local-db ":memory:"))
  
  (print "[ IN MEMORY TEST ] STARTING CREATING TABLE ...")
  
  (print "[ IN MEMORY TEST ] ENDING CREATING TABLE ...")

  (print "...")

  (print "[ IN MEMORY TEST ] INSERTING VALUES INTO TABLE ...")

  (print "[ IN MEMORY TEST ] COMPLETE INSERTING VALUES INTO TABLE ...")

  (print "...")

  (print "[ IN MEMORY TEST ] QUERYING TABLE FOR VALUES ...")

  (print "[ IN MEMORY TEST ] COMPLETE QUERYING TABLE FOR VALUES ...")

  (print "...")

  (print "[ IN MEMORY TEST ] CLOSING DATABASE CONNECTION")

  (db/close-db in-memory-db)

  (print "[ IN MEMORY TEST ] COMPLETE CLOSING DATABASE CONNECTION")

  (print "...")
  
  (print "[ IN MEMORY TEST ] ENDING IN MEMORY TESTING ..."))
