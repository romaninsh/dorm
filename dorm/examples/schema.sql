SET client_min_messages TO WARNING;

DROP TABLE IF EXISTS order_line;
DROP TABLE IF EXISTS "order";
DROP TABLE IF EXISTS product;
DROP TABLE IF EXISTS client;

CREATE TABLE product (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    default_price DECIMAL(10, 2) NOT NULL
);

CREATE TABLE client (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) UNIQUE,
    phone VARCHAR(50),
    is_vip BOOLEAN DEFAULT FALSE,
    address TEXT
);

CREATE TABLE "order" (
    id SERIAL PRIMARY KEY,
    client_id INTEGER NOT NULL,
    order_date TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    status VARCHAR(100) DEFAULT 'Pending',
    CONSTRAINT fk_client
        FOREIGN KEY (client_id)
        REFERENCES client (id)
        ON DELETE CASCADE
);

CREATE TABLE order_line (
    id SERIAL PRIMARY KEY,
    order_id INTEGER NOT NULL,
    product_id INTEGER NOT NULL,
    quantity INTEGER NOT NULL,
    price_per_unit DECIMAL(10, 2) NOT NULL,
    CONSTRAINT fk_order
        FOREIGN KEY (order_id)
        REFERENCES "order" (id)
        ON DELETE CASCADE,
    CONSTRAINT fk_product
        FOREIGN KEY (product_id)
        REFERENCES product (id)
        ON DELETE CASCADE
);

-- Insert products with futuristic descriptions
INSERT INTO product (name, description, default_price) VALUES
('Flux Capacitor', 'Essential for any time travel vehicle, enables you to escape the 50s.', 299.99),
('Hoverboard', 'Futuristic skateboard that floats on air. Great for escaping foes.', 199.99),
('Mr. Fusion Home Energy Reactor', 'Converts household waste to power for time travel.', 399.99),
('Sports Almanac', 'Includes sports statistics from the future; betting has never been easier.', 99.99),
('Plutonium', 'Provides the necessary gigawatts of electricity to power the Flux Capacitor.', 499.99),
('Auto-adjusting Jacket', 'Stylish, self-drying jacket, adjusts to fit all sizes. Perfect for sudden weather changes.', 89.99),
('Self-lacing Shoes', 'Saves time in the morning. Never trip over a lace again!', 149.99),
('Hydrator', 'Instantly hydrates food. Pizza in just seconds!', 159.99),
('Time Circuit', 'Tracks and displays the time travel destination. Don’t leave home without it.', 259.99),
('Flying Car Upgrade Kit', 'Converts any car into a flying car. Avoid traffic forever.', 10000.99),
('Portable Hover Conversion', 'Apply hover capabilities to anything. Why walk when you can float?', 999.99),
('Fusion Diagnostics Tool', 'Ensures your Mr. Fusion is running smoothly. Avoid untimely breakdowns.', 249.99),
('Holographic Hat', 'Fashion from the future. Includes mood lighting and display messages.', 49.99),
('Rope', 'Great at saving you from the approaching vehicle, can be used with hovercars.', 19.99),
('Bionic Gloves', 'Gives you the grip strength of a robot. Handy for tough repairs.', 75.99);

-- Insert clients
INSERT INTO client (name, email, phone, is_vip, address) VALUES
('Marty McFly', 'marty.mcfly@example.com', '555-1955', TRUE, '9303 Lyon Drive Hill Valley'),
('Doc Brown', 'doc.brown@example.com', '555-1885', TRUE, '1640 Riverside Drive'),
('Biff Tannen', 'biff.tannen@example.com', '555-1955', FALSE, '1809 Mason St');

-- Insert orders
INSERT INTO "order" (client_id, order_date, status) VALUES
(1, NOW(), 'Pending'),
(2, NOW(), 'Shipped'),
(1, NOW() - INTERVAL '1 day', 'Delivered'),
(2, NOW() - INTERVAL '2 days', 'Pending'),
(3, NOW() - INTERVAL '3 days', 'Cancelled');

-- Insert order lines
INSERT INTO order_line (order_id, product_id, quantity, price_per_unit) VALUES
(1, 1, 1, 299.99),
(1, 2, 1, 199.99),
(1, 3, 1, 399.99),
(2, 4, 2, 99.99),
(2, 5, 2, 499.99),
(3, 6, 1, 89.99),
(3, 7, 1, 149.99),
(3, 8, 1, 159.99),
(4, 9, 1, 259.99),
(4, 10, 1, 10000.99),
(4, 11, 1, 999.99),
(5, 12, 1, 249.99),
(5, 13, 1, 49.99),
(5, 14, 1, 19.99),
(1, 15, 1, 75.99);
