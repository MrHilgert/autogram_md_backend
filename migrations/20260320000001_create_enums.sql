CREATE TYPE fuel_type AS ENUM ('petrol','diesel','gas_methane','gas_propane','petrol_gas_methane','petrol_gas_propane','electric','hybrid');
CREATE TYPE body_type AS ENUM ('sedan','hatchback','wagon','suv','coupe','minivan','pickup','convertible','van');
CREATE TYPE transmission_type AS ENUM ('manual','automatic','cvt','robot');
CREATE TYPE drive_type AS ENUM ('fwd','rwd','awd');
CREATE TYPE steering_side AS ENUM ('left','right');
CREATE TYPE car_condition AS ENUM ('new','used');
CREATE TYPE listing_status AS ENUM ('active','sold','archived');
CREATE TYPE currency_code AS ENUM ('USD','EUR','RUP');
