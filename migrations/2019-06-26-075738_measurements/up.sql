CREATE TABLE measurements
(
    id          INTEGER PRIMARY KEY AUTO_INCREMENT NOT NULL,
    time        DATETIME                           NOT NULL,
    sensor      TEXT                               NOT NULL,
    temperature FLOAT                              NOT NULL,
    humidity    FLOAT                              NOT NULL,
    pressure    FLOAT                              NOT NULL
)