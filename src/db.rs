

pub fn getDbConnection() -> Result<Connection, Error> {
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    Connection::connect(db_url, TlsMode::None)
}

pub fn setupDb() {
    let conn = getDbConnection().unwrap();

    conn.execute("CREATE TABLE IF NOT EXISTS visits (
        id SERIAL PRIMARY KEY,
        host VARCHAR NOT NULL,
        page_url VARCHAR NOT NULL,
        country VARCHAR NOT NULL,
        anonymous_id VARCHAR NOT NULL,
        visited_at TIMESTAMP NOT NULL,
        window_Width INT NOT NULL,
        is_pwa_mode BOOLEAN NOT NULL,
        user_agent VARCHAR NOT NULL,
        browser_version VARCHAR NOT NULL,
        browser_name VARCHAR NOT NULL,
        os_version VARCHAR NOT NULL,
        os_name VARCHAR NOT NULL,
        is_mobile BOOLEAN NOT NULL,
        is_bot BOOLEAN NOT NULL
    )", &[]).unwrap();

    conn.execute("CREATE TABLE IF NOT EXISTS navigations (
        id SERIAL PRIMARY KEY,
        page_url VARCHAR NOT NULL,
        rank INT NOT NULL,
        time_spent INT NOT NULL,
        visit_id INT NOT NULL,
        FOREIGN KEY (visit_id) REFERENCES visits(id)
    )", &[]).unwrap();

    // create table for countries
    conn.execute("CREATE TABLE IF NOT EXISTS countries (
        id SERIAL PRIMARY KEY,
        name VARCHAR NOT NULL,
        iso_code VARCHAR NOT NULL,
        is_eu BOOLEAN NOT NULL
    )", &[]).unwrap();

    // create table for ip ranges
    conn.execute("CREATE TABLE IF NOT EXISTS ipv4_ranges (
        id SERIAL PRIMARY KEY,
        start_ip INT NOT NULL,
        end_ip INT NOT NULL,
        country_id INT NOT NULL,
        FOREIGN KEY (country_id) REFERENCES countries(id)
    )", &[]).unwrap();

    conn.execute("CREATE TABLE IF NOT EXISTS ipv6_ranges (
        id SERIAL PRIMARY KEY,
        start_ip VARCHAR NOT NULL,
        end_ip VARCHAR NOT NULL,
        country_id INT NOT NULL,
        FOREIGN KEY (country_id) REFERENCES countries(id)
    )
    ", &[]).unwrap();

    // clear countries table before filling
    conn.execute("DELETE FROM countries", &[]).unwrap();
    // fill countries table
    fillCountries();

    // clear ip ranges table before filling
    conn.execute("DELETE FROM ipv4_ranges", &[]).unwrap();
    conn.execute("DELETE FROM ipv6_ranges", &[]).unwrap();
    // fill ip ranges table
    fillIpRanges();
}

fn fillIpRanges() {
    let conn = getDbConnection().unwrap();

    let mut reader = csv::Reader::from_path("data/ipv4_ranges.csv").unwrap();
    for record in reader.records() {
        let record = record.unwrap();
        let start_ip = record[0].parse::<i32>().unwrap();
        let end_ip = record[1].parse::<i32>().unwrap();
        let country_id = record[2].parse::<i32>().unwrap();

        conn.execute("INSERT INTO ipv4_ranges (start_ip, end_ip, country_id) VALUES ($1, $2, $3)",
            &[&start_ip, &end_ip, &country_id]).unwrap();
    }

    let mut reader = csv::Reader::from_path("data/ipv6_ranges.csv").unwrap();
    for record in reader.records() {
        let record = record.unwrap();
        let start_ip = record[0].to_string();
        let end_ip = record[1].to_string();
        let country_id = record[2].parse::<i32>().unwrap();

        conn.execute("INSERT INTO ipv6_ranges (start_ip, end_ip, country_id) VALUES ($1, $2, $3)",
            &[&start_ip, &end_ip, &country_id]).unwrap();
    }
}

// fill countries from csv file
fn fillCountries() {
    let conn = getDbConnection().unwrap();

    let mut reader = csv::Reader::from_path("data/countries.csv").unwrap();
    for record in reader.records() {
        let record = record.unwrap();
        let name = record[0].to_string();
        let iso_code = record[1].to_string();
        let is_eu = record[2].parse::<bool>().unwrap();

        conn.execute("INSERT INTO countries (name, iso_code, is_eu) VALUES ($1, $2, $3)",
            &[&name, &iso_code, &is_eu]).unwrap();
    }
}