create table if not exists public.measurements
(
    id           serial
        constraint measurements_pk primary key,
    sensor_id    text      not null,
    firmware     text      not null,
    timestamp    timestamp not null,
    temperature  float4    not null,
    rel_humidity float4    not null,
    abs_humidity float4    not null,
    pressure     float4    not null,
    raw_voltage  integer   not null,
    charge       float4    not null,
    constraint measurements_uk unique (sensor_id, timestamp)
);

alter table public.measurements
    owner to weather_station;

