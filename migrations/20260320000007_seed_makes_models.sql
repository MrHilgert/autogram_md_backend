-- Seed: popular car makes and models for Moldova/PMR/CIS market

-- ============================================================
-- Makes
-- ============================================================
INSERT INTO car_makes (name, slug) VALUES
    ('Audi',          'audi'),
    ('BMW',           'bmw'),
    ('Mercedes-Benz', 'mercedes-benz'),
    ('Volkswagen',    'volkswagen'),
    ('Toyota',        'toyota'),
    ('Renault',       'renault'),
    ('Dacia',         'dacia'),
    ('Skoda',         'skoda'),
    ('Honda',         'honda'),
    ('Hyundai',       'hyundai'),
    ('KIA',           'kia'),
    ('Nissan',        'nissan'),
    ('Ford',          'ford'),
    ('Chevrolet',     'chevrolet'),
    ('Opel',          'opel'),
    ('Peugeot',       'peugeot'),
    ('Citroen',       'citroen'),
    ('Fiat',          'fiat'),
    ('Mazda',         'mazda'),
    ('Mitsubishi',    'mitsubishi'),
    ('Subaru',        'subaru'),
    ('Suzuki',        'suzuki'),
    ('Volvo',         'volvo'),
    ('Lexus',         'lexus'),
    ('Land Rover',    'land-rover'),
    ('Jeep',          'jeep'),
    ('Porsche',       'porsche'),
    ('Lada (ВАЗ)',    'lada'),
    ('Dodge',         'dodge'),
    ('Infiniti',      'infiniti'),
    ('Seat',          'seat'),
    ('Daewoo',        'daewoo'),
    ('Geely',         'geely'),
    ('Chery',         'chery')
ON CONFLICT (name) DO NOTHING;

-- ============================================================
-- Models
-- ============================================================

-- 1. Audi
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'audi'), 'A3',     'a3'),
    ((SELECT id FROM car_makes WHERE slug = 'audi'), 'A4',     'a4'),
    ((SELECT id FROM car_makes WHERE slug = 'audi'), 'A5',     'a5'),
    ((SELECT id FROM car_makes WHERE slug = 'audi'), 'A6',     'a6'),
    ((SELECT id FROM car_makes WHERE slug = 'audi'), 'A7',     'a7'),
    ((SELECT id FROM car_makes WHERE slug = 'audi'), 'A8',     'a8'),
    ((SELECT id FROM car_makes WHERE slug = 'audi'), 'Q3',     'q3'),
    ((SELECT id FROM car_makes WHERE slug = 'audi'), 'Q5',     'q5'),
    ((SELECT id FROM car_makes WHERE slug = 'audi'), 'Q7',     'q7'),
    ((SELECT id FROM car_makes WHERE slug = 'audi'), 'Q8',     'q8'),
    ((SELECT id FROM car_makes WHERE slug = 'audi'), 'TT',     'tt'),
    ((SELECT id FROM car_makes WHERE slug = 'audi'), 'e-tron', 'e-tron')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 2. BMW
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'bmw'), '1 Series', '1-series'),
    ((SELECT id FROM car_makes WHERE slug = 'bmw'), '2 Series', '2-series'),
    ((SELECT id FROM car_makes WHERE slug = 'bmw'), '3 Series', '3-series'),
    ((SELECT id FROM car_makes WHERE slug = 'bmw'), '4 Series', '4-series'),
    ((SELECT id FROM car_makes WHERE slug = 'bmw'), '5 Series', '5-series'),
    ((SELECT id FROM car_makes WHERE slug = 'bmw'), '6 Series', '6-series'),
    ((SELECT id FROM car_makes WHERE slug = 'bmw'), '7 Series', '7-series'),
    ((SELECT id FROM car_makes WHERE slug = 'bmw'), 'X1',       'x1'),
    ((SELECT id FROM car_makes WHERE slug = 'bmw'), 'X3',       'x3'),
    ((SELECT id FROM car_makes WHERE slug = 'bmw'), 'X5',       'x5'),
    ((SELECT id FROM car_makes WHERE slug = 'bmw'), 'X6',       'x6'),
    ((SELECT id FROM car_makes WHERE slug = 'bmw'), 'X7',       'x7')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 3. Mercedes-Benz
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'mercedes-benz'), 'A-Class', 'a-class'),
    ((SELECT id FROM car_makes WHERE slug = 'mercedes-benz'), 'B-Class', 'b-class'),
    ((SELECT id FROM car_makes WHERE slug = 'mercedes-benz'), 'C-Class', 'c-class'),
    ((SELECT id FROM car_makes WHERE slug = 'mercedes-benz'), 'E-Class', 'e-class'),
    ((SELECT id FROM car_makes WHERE slug = 'mercedes-benz'), 'S-Class', 's-class'),
    ((SELECT id FROM car_makes WHERE slug = 'mercedes-benz'), 'GLA',     'gla'),
    ((SELECT id FROM car_makes WHERE slug = 'mercedes-benz'), 'GLC',     'glc'),
    ((SELECT id FROM car_makes WHERE slug = 'mercedes-benz'), 'GLE',     'gle'),
    ((SELECT id FROM car_makes WHERE slug = 'mercedes-benz'), 'GLS',     'gls'),
    ((SELECT id FROM car_makes WHERE slug = 'mercedes-benz'), 'CLA',     'cla'),
    ((SELECT id FROM car_makes WHERE slug = 'mercedes-benz'), 'CLS',     'cls'),
    ((SELECT id FROM car_makes WHERE slug = 'mercedes-benz'), 'G-Class', 'g-class')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 4. Volkswagen
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'volkswagen'), 'Golf',        'golf'),
    ((SELECT id FROM car_makes WHERE slug = 'volkswagen'), 'Passat',      'passat'),
    ((SELECT id FROM car_makes WHERE slug = 'volkswagen'), 'Polo',        'polo'),
    ((SELECT id FROM car_makes WHERE slug = 'volkswagen'), 'Tiguan',      'tiguan'),
    ((SELECT id FROM car_makes WHERE slug = 'volkswagen'), 'Touareg',     'touareg'),
    ((SELECT id FROM car_makes WHERE slug = 'volkswagen'), 'Jetta',       'jetta'),
    ((SELECT id FROM car_makes WHERE slug = 'volkswagen'), 'Arteon',      'arteon'),
    ((SELECT id FROM car_makes WHERE slug = 'volkswagen'), 'T-Roc',       't-roc'),
    ((SELECT id FROM car_makes WHERE slug = 'volkswagen'), 'ID.3',        'id-3'),
    ((SELECT id FROM car_makes WHERE slug = 'volkswagen'), 'ID.4',        'id-4'),
    ((SELECT id FROM car_makes WHERE slug = 'volkswagen'), 'Caddy',       'caddy'),
    ((SELECT id FROM car_makes WHERE slug = 'volkswagen'), 'Transporter', 'transporter')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 5. Toyota
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'toyota'), 'Camry',        'camry'),
    ((SELECT id FROM car_makes WHERE slug = 'toyota'), 'Corolla',      'corolla'),
    ((SELECT id FROM car_makes WHERE slug = 'toyota'), 'RAV4',         'rav4'),
    ((SELECT id FROM car_makes WHERE slug = 'toyota'), 'Land Cruiser', 'land-cruiser'),
    ((SELECT id FROM car_makes WHERE slug = 'toyota'), 'Prado',        'prado'),
    ((SELECT id FROM car_makes WHERE slug = 'toyota'), 'Hilux',        'hilux'),
    ((SELECT id FROM car_makes WHERE slug = 'toyota'), 'Yaris',        'yaris'),
    ((SELECT id FROM car_makes WHERE slug = 'toyota'), 'Auris',        'auris'),
    ((SELECT id FROM car_makes WHERE slug = 'toyota'), 'Avensis',      'avensis'),
    ((SELECT id FROM car_makes WHERE slug = 'toyota'), 'C-HR',         'c-hr'),
    ((SELECT id FROM car_makes WHERE slug = 'toyota'), 'Supra',        'supra')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 6. Renault
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'renault'), 'Logan',    'logan'),
    ((SELECT id FROM car_makes WHERE slug = 'renault'), 'Sandero',  'sandero'),
    ((SELECT id FROM car_makes WHERE slug = 'renault'), 'Duster',   'duster'),
    ((SELECT id FROM car_makes WHERE slug = 'renault'), 'Megane',   'megane'),
    ((SELECT id FROM car_makes WHERE slug = 'renault'), 'Clio',     'clio'),
    ((SELECT id FROM car_makes WHERE slug = 'renault'), 'Captur',   'captur'),
    ((SELECT id FROM car_makes WHERE slug = 'renault'), 'Kadjar',   'kadjar'),
    ((SELECT id FROM car_makes WHERE slug = 'renault'), 'Scenic',   'scenic'),
    ((SELECT id FROM car_makes WHERE slug = 'renault'), 'Koleos',   'koleos'),
    ((SELECT id FROM car_makes WHERE slug = 'renault'), 'Talisman', 'talisman')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 7. Dacia
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'dacia'), 'Logan',   'logan'),
    ((SELECT id FROM car_makes WHERE slug = 'dacia'), 'Sandero', 'sandero'),
    ((SELECT id FROM car_makes WHERE slug = 'dacia'), 'Duster',  'duster'),
    ((SELECT id FROM car_makes WHERE slug = 'dacia'), 'Spring',  'spring'),
    ((SELECT id FROM car_makes WHERE slug = 'dacia'), 'Jogger',  'jogger')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 8. Skoda
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'skoda'), 'Octavia', 'octavia'),
    ((SELECT id FROM car_makes WHERE slug = 'skoda'), 'Superb',  'superb'),
    ((SELECT id FROM car_makes WHERE slug = 'skoda'), 'Fabia',   'fabia'),
    ((SELECT id FROM car_makes WHERE slug = 'skoda'), 'Rapid',   'rapid'),
    ((SELECT id FROM car_makes WHERE slug = 'skoda'), 'Kodiaq',  'kodiaq'),
    ((SELECT id FROM car_makes WHERE slug = 'skoda'), 'Karoq',   'karoq'),
    ((SELECT id FROM car_makes WHERE slug = 'skoda'), 'Kamiq',   'kamiq'),
    ((SELECT id FROM car_makes WHERE slug = 'skoda'), 'Scala',   'scala')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 9. Honda
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'honda'), 'Civic',  'civic'),
    ((SELECT id FROM car_makes WHERE slug = 'honda'), 'Accord', 'accord'),
    ((SELECT id FROM car_makes WHERE slug = 'honda'), 'CR-V',   'cr-v'),
    ((SELECT id FROM car_makes WHERE slug = 'honda'), 'HR-V',   'hr-v'),
    ((SELECT id FROM car_makes WHERE slug = 'honda'), 'Jazz',   'jazz'),
    ((SELECT id FROM car_makes WHERE slug = 'honda'), 'Pilot',  'pilot')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 10. Hyundai
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'hyundai'), 'Tucson',   'tucson'),
    ((SELECT id FROM car_makes WHERE slug = 'hyundai'), 'Santa Fe', 'santa-fe'),
    ((SELECT id FROM car_makes WHERE slug = 'hyundai'), 'Elantra',  'elantra'),
    ((SELECT id FROM car_makes WHERE slug = 'hyundai'), 'Sonata',   'sonata'),
    ((SELECT id FROM car_makes WHERE slug = 'hyundai'), 'i30',      'i30'),
    ((SELECT id FROM car_makes WHERE slug = 'hyundai'), 'i20',      'i20'),
    ((SELECT id FROM car_makes WHERE slug = 'hyundai'), 'Kona',     'kona'),
    ((SELECT id FROM car_makes WHERE slug = 'hyundai'), 'Creta',    'creta'),
    ((SELECT id FROM car_makes WHERE slug = 'hyundai'), 'Accent',   'accent')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 11. KIA
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'kia'), 'Sportage', 'sportage'),
    ((SELECT id FROM car_makes WHERE slug = 'kia'), 'Ceed',     'ceed'),
    ((SELECT id FROM car_makes WHERE slug = 'kia'), 'Sorento',  'sorento'),
    ((SELECT id FROM car_makes WHERE slug = 'kia'), 'Rio',      'rio'),
    ((SELECT id FROM car_makes WHERE slug = 'kia'), 'Seltos',   'seltos'),
    ((SELECT id FROM car_makes WHERE slug = 'kia'), 'Optima',   'optima'),
    ((SELECT id FROM car_makes WHERE slug = 'kia'), 'Stinger',  'stinger'),
    ((SELECT id FROM car_makes WHERE slug = 'kia'), 'Carnival', 'carnival')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 12. Nissan
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'nissan'), 'Qashqai',    'qashqai'),
    ((SELECT id FROM car_makes WHERE slug = 'nissan'), 'X-Trail',    'x-trail'),
    ((SELECT id FROM car_makes WHERE slug = 'nissan'), 'Juke',       'juke'),
    ((SELECT id FROM car_makes WHERE slug = 'nissan'), 'Leaf',       'leaf'),
    ((SELECT id FROM car_makes WHERE slug = 'nissan'), 'Micra',      'micra'),
    ((SELECT id FROM car_makes WHERE slug = 'nissan'), 'Note',       'note'),
    ((SELECT id FROM car_makes WHERE slug = 'nissan'), 'Navara',     'navara'),
    ((SELECT id FROM car_makes WHERE slug = 'nissan'), 'Pathfinder', 'pathfinder'),
    ((SELECT id FROM car_makes WHERE slug = 'nissan'), 'Almera',     'almera')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 13. Ford
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'ford'), 'Focus',    'focus'),
    ((SELECT id FROM car_makes WHERE slug = 'ford'), 'Fiesta',   'fiesta'),
    ((SELECT id FROM car_makes WHERE slug = 'ford'), 'Mondeo',   'mondeo'),
    ((SELECT id FROM car_makes WHERE slug = 'ford'), 'Kuga',     'kuga'),
    ((SELECT id FROM car_makes WHERE slug = 'ford'), 'EcoSport', 'ecosport'),
    ((SELECT id FROM car_makes WHERE slug = 'ford'), 'Ranger',   'ranger'),
    ((SELECT id FROM car_makes WHERE slug = 'ford'), 'Explorer', 'explorer'),
    ((SELECT id FROM car_makes WHERE slug = 'ford'), 'Mustang',  'mustang'),
    ((SELECT id FROM car_makes WHERE slug = 'ford'), 'Transit',  'transit')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 14. Chevrolet
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'chevrolet'), 'Aveo',    'aveo'),
    ((SELECT id FROM car_makes WHERE slug = 'chevrolet'), 'Cruze',   'cruze'),
    ((SELECT id FROM car_makes WHERE slug = 'chevrolet'), 'Malibu',  'malibu'),
    ((SELECT id FROM car_makes WHERE slug = 'chevrolet'), 'Captiva', 'captiva'),
    ((SELECT id FROM car_makes WHERE slug = 'chevrolet'), 'Spark',   'spark'),
    ((SELECT id FROM car_makes WHERE slug = 'chevrolet'), 'Orlando', 'orlando'),
    ((SELECT id FROM car_makes WHERE slug = 'chevrolet'), 'Tahoe',   'tahoe'),
    ((SELECT id FROM car_makes WHERE slug = 'chevrolet'), 'Camaro',  'camaro')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 15. Opel
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'opel'), 'Astra',     'astra'),
    ((SELECT id FROM car_makes WHERE slug = 'opel'), 'Corsa',     'corsa'),
    ((SELECT id FROM car_makes WHERE slug = 'opel'), 'Insignia',  'insignia'),
    ((SELECT id FROM car_makes WHERE slug = 'opel'), 'Mokka',     'mokka'),
    ((SELECT id FROM car_makes WHERE slug = 'opel'), 'Crossland', 'crossland'),
    ((SELECT id FROM car_makes WHERE slug = 'opel'), 'Grandland', 'grandland'),
    ((SELECT id FROM car_makes WHERE slug = 'opel'), 'Zafira',    'zafira')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 16. Peugeot
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'peugeot'), '208',     '208'),
    ((SELECT id FROM car_makes WHERE slug = 'peugeot'), '308',     '308'),
    ((SELECT id FROM car_makes WHERE slug = 'peugeot'), '3008',    '3008'),
    ((SELECT id FROM car_makes WHERE slug = 'peugeot'), '508',     '508'),
    ((SELECT id FROM car_makes WHERE slug = 'peugeot'), '2008',    '2008'),
    ((SELECT id FROM car_makes WHERE slug = 'peugeot'), '5008',    '5008'),
    ((SELECT id FROM car_makes WHERE slug = 'peugeot'), 'Partner', 'partner')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 17. Citroen
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'citroen'), 'C3',           'c3'),
    ((SELECT id FROM car_makes WHERE slug = 'citroen'), 'C4',           'c4'),
    ((SELECT id FROM car_makes WHERE slug = 'citroen'), 'C5 Aircross',  'c5-aircross'),
    ((SELECT id FROM car_makes WHERE slug = 'citroen'), 'Berlingo',     'berlingo'),
    ((SELECT id FROM car_makes WHERE slug = 'citroen'), 'C-Elysee',     'c-elysee')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 18. Fiat
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'fiat'), 'Punto', 'punto'),
    ((SELECT id FROM car_makes WHERE slug = 'fiat'), '500',   '500'),
    ((SELECT id FROM car_makes WHERE slug = 'fiat'), 'Panda', 'panda'),
    ((SELECT id FROM car_makes WHERE slug = 'fiat'), 'Tipo',  'tipo'),
    ((SELECT id FROM car_makes WHERE slug = 'fiat'), 'Doblo', 'doblo')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 19. Mazda
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'mazda'), '3',     '3'),
    ((SELECT id FROM car_makes WHERE slug = 'mazda'), '6',     '6'),
    ((SELECT id FROM car_makes WHERE slug = 'mazda'), 'CX-3',  'cx-3'),
    ((SELECT id FROM car_makes WHERE slug = 'mazda'), 'CX-5',  'cx-5'),
    ((SELECT id FROM car_makes WHERE slug = 'mazda'), 'CX-30', 'cx-30'),
    ((SELECT id FROM car_makes WHERE slug = 'mazda'), 'MX-5',  'mx-5')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 20. Mitsubishi
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'mitsubishi'), 'Outlander',     'outlander'),
    ((SELECT id FROM car_makes WHERE slug = 'mitsubishi'), 'Lancer',        'lancer'),
    ((SELECT id FROM car_makes WHERE slug = 'mitsubishi'), 'ASX',           'asx'),
    ((SELECT id FROM car_makes WHERE slug = 'mitsubishi'), 'Pajero',        'pajero'),
    ((SELECT id FROM car_makes WHERE slug = 'mitsubishi'), 'L200',          'l200'),
    ((SELECT id FROM car_makes WHERE slug = 'mitsubishi'), 'Eclipse Cross', 'eclipse-cross')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 21. Subaru
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'subaru'), 'Forester', 'forester'),
    ((SELECT id FROM car_makes WHERE slug = 'subaru'), 'Outback',  'outback'),
    ((SELECT id FROM car_makes WHERE slug = 'subaru'), 'XV',       'xv'),
    ((SELECT id FROM car_makes WHERE slug = 'subaru'), 'Impreza',  'impreza'),
    ((SELECT id FROM car_makes WHERE slug = 'subaru'), 'Legacy',   'legacy'),
    ((SELECT id FROM car_makes WHERE slug = 'subaru'), 'WRX',      'wrx')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 22. Suzuki
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'suzuki'), 'Vitara', 'vitara'),
    ((SELECT id FROM car_makes WHERE slug = 'suzuki'), 'SX4',    'sx4'),
    ((SELECT id FROM car_makes WHERE slug = 'suzuki'), 'Swift',  'swift'),
    ((SELECT id FROM car_makes WHERE slug = 'suzuki'), 'Jimny',  'jimny'),
    ((SELECT id FROM car_makes WHERE slug = 'suzuki'), 'Ignis',  'ignis')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 23. Volvo
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'volvo'), 'XC40', 'xc40'),
    ((SELECT id FROM car_makes WHERE slug = 'volvo'), 'XC60', 'xc60'),
    ((SELECT id FROM car_makes WHERE slug = 'volvo'), 'XC90', 'xc90'),
    ((SELECT id FROM car_makes WHERE slug = 'volvo'), 'S60',  's60'),
    ((SELECT id FROM car_makes WHERE slug = 'volvo'), 'S90',  's90'),
    ((SELECT id FROM car_makes WHERE slug = 'volvo'), 'V60',  'v60'),
    ((SELECT id FROM car_makes WHERE slug = 'volvo'), 'V90',  'v90')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 24. Lexus
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'lexus'), 'IS', 'is'),
    ((SELECT id FROM car_makes WHERE slug = 'lexus'), 'ES', 'es'),
    ((SELECT id FROM car_makes WHERE slug = 'lexus'), 'LS', 'ls'),
    ((SELECT id FROM car_makes WHERE slug = 'lexus'), 'NX', 'nx'),
    ((SELECT id FROM car_makes WHERE slug = 'lexus'), 'RX', 'rx'),
    ((SELECT id FROM car_makes WHERE slug = 'lexus'), 'UX', 'ux'),
    ((SELECT id FROM car_makes WHERE slug = 'lexus'), 'GX', 'gx'),
    ((SELECT id FROM car_makes WHERE slug = 'lexus'), 'LX', 'lx')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 25. Land Rover
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'land-rover'), 'Range Rover',        'range-rover'),
    ((SELECT id FROM car_makes WHERE slug = 'land-rover'), 'Range Rover Sport',  'range-rover-sport'),
    ((SELECT id FROM car_makes WHERE slug = 'land-rover'), 'Range Rover Evoque', 'range-rover-evoque'),
    ((SELECT id FROM car_makes WHERE slug = 'land-rover'), 'Discovery',          'discovery'),
    ((SELECT id FROM car_makes WHERE slug = 'land-rover'), 'Defender',           'defender'),
    ((SELECT id FROM car_makes WHERE slug = 'land-rover'), 'Freelander',         'freelander')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 26. Jeep
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'jeep'), 'Wrangler',       'wrangler'),
    ((SELECT id FROM car_makes WHERE slug = 'jeep'), 'Grand Cherokee', 'grand-cherokee'),
    ((SELECT id FROM car_makes WHERE slug = 'jeep'), 'Cherokee',       'cherokee'),
    ((SELECT id FROM car_makes WHERE slug = 'jeep'), 'Compass',        'compass'),
    ((SELECT id FROM car_makes WHERE slug = 'jeep'), 'Renegade',       'renegade')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 27. Porsche
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'porsche'), 'Cayenne',  'cayenne'),
    ((SELECT id FROM car_makes WHERE slug = 'porsche'), 'Macan',    'macan'),
    ((SELECT id FROM car_makes WHERE slug = 'porsche'), 'Panamera', 'panamera'),
    ((SELECT id FROM car_makes WHERE slug = 'porsche'), '911',      '911'),
    ((SELECT id FROM car_makes WHERE slug = 'porsche'), 'Taycan',   'taycan'),
    ((SELECT id FROM car_makes WHERE slug = 'porsche'), 'Boxster',  'boxster')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 28. Lada (ВАЗ)
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'lada'), 'Granta', 'granta'),
    ((SELECT id FROM car_makes WHERE slug = 'lada'), 'Vesta',  'vesta'),
    ((SELECT id FROM car_makes WHERE slug = 'lada'), 'XRAY',   'xray'),
    ((SELECT id FROM car_makes WHERE slug = 'lada'), 'Niva',   'niva'),
    ((SELECT id FROM car_makes WHERE slug = 'lada'), 'Largus', 'largus'),
    ((SELECT id FROM car_makes WHERE slug = 'lada'), 'Priora', 'priora'),
    ((SELECT id FROM car_makes WHERE slug = 'lada'), 'Kalina', 'kalina'),
    ((SELECT id FROM car_makes WHERE slug = 'lada'), '2107',   '2107'),
    ((SELECT id FROM car_makes WHERE slug = 'lada'), '2109',   '2109'),
    ((SELECT id FROM car_makes WHERE slug = 'lada'), '2110',   '2110'),
    ((SELECT id FROM car_makes WHERE slug = 'lada'), '2114',   '2114')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 29. Dodge
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'dodge'), 'Charger',    'charger'),
    ((SELECT id FROM car_makes WHERE slug = 'dodge'), 'Challenger', 'challenger'),
    ((SELECT id FROM car_makes WHERE slug = 'dodge'), 'Durango',    'durango'),
    ((SELECT id FROM car_makes WHERE slug = 'dodge'), 'RAM',        'ram')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 30. Infiniti
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'infiniti'), 'Q50',  'q50'),
    ((SELECT id FROM car_makes WHERE slug = 'infiniti'), 'Q60',  'q60'),
    ((SELECT id FROM car_makes WHERE slug = 'infiniti'), 'QX50', 'qx50'),
    ((SELECT id FROM car_makes WHERE slug = 'infiniti'), 'QX60', 'qx60'),
    ((SELECT id FROM car_makes WHERE slug = 'infiniti'), 'QX80', 'qx80')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 31. Seat
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'seat'), 'Leon',    'leon'),
    ((SELECT id FROM car_makes WHERE slug = 'seat'), 'Ibiza',   'ibiza'),
    ((SELECT id FROM car_makes WHERE slug = 'seat'), 'Ateca',   'ateca'),
    ((SELECT id FROM car_makes WHERE slug = 'seat'), 'Arona',   'arona'),
    ((SELECT id FROM car_makes WHERE slug = 'seat'), 'Tarraco', 'tarraco')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 32. Daewoo
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'daewoo'), 'Matiz',   'matiz'),
    ((SELECT id FROM car_makes WHERE slug = 'daewoo'), 'Nexia',   'nexia'),
    ((SELECT id FROM car_makes WHERE slug = 'daewoo'), 'Lanos',   'lanos'),
    ((SELECT id FROM car_makes WHERE slug = 'daewoo'), 'Nubira',  'nubira'),
    ((SELECT id FROM car_makes WHERE slug = 'daewoo'), 'Leganza', 'leganza')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 33. Geely
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'geely'), 'Atlas',   'atlas'),
    ((SELECT id FROM car_makes WHERE slug = 'geely'), 'Coolray', 'coolray'),
    ((SELECT id FROM car_makes WHERE slug = 'geely'), 'Emgrand', 'emgrand'),
    ((SELECT id FROM car_makes WHERE slug = 'geely'), 'Tugella', 'tugella')
ON CONFLICT (make_id, slug) DO NOTHING;

-- 34. Chery
INSERT INTO car_models (make_id, name, slug) VALUES
    ((SELECT id FROM car_makes WHERE slug = 'chery'), 'Tiggo 4',     'tiggo-4'),
    ((SELECT id FROM car_makes WHERE slug = 'chery'), 'Tiggo 7 Pro', 'tiggo-7-pro'),
    ((SELECT id FROM car_makes WHERE slug = 'chery'), 'Tiggo 8',     'tiggo-8'),
    ((SELECT id FROM car_makes WHERE slug = 'chery'), 'Arrizo',      'arrizo')
ON CONFLICT (make_id, slug) DO NOTHING;
