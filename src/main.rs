mod db;

fn main() {
    // set up database
    db::setupDb();

    // create api server
    let mut api = Nickel::new();

    // set up routes
    api.put("/api/visit", middleware! { |request, response|
        let conn = db::getDbConnection().unwrap();
        let mut visit = Visit::new();
        // get host from headers
        visit.host = request.origin.headers.get::<Host>().unwrap().hostname.to_string();
        visit.page_url = request.param("page_url").unwrap().to_string();
        // get country from ip
        visit.country = getCountryFromIp(request.origin.remote_addr.ip().to_string()).unwrap();
        // create anonymous id from ip, user agent and date
        visit.anonymous_id = getAnonymousId(request.origin.remote_addr.ip().to_string(), request.param("user_agent").unwrap().to_string());
        // current date
        visit.visited_at = Utc::now().naive_utc();
        visit.window_width = request.param("window_width").unwrap().to_string().parse::<i32>().unwrap();
        visit.is_pwa_mode = request.param("is_pwa_mode").unwrap().to_string().parse::<bool>().unwrap();
        // get user-agent from headers
        visit.user_agent = request.origin.headers.get::<UserAgent>().unwrap().to_string();
        visit.browser_version = request.origin.headers.get::<UserAgent>().unwrap().version.unwrap().to_string();
        visit.browser_name = request.origin.headers.get::<UserAgent>().unwrap().product.unwrap().to_string();
        visit.os_version = request.origin.headers.get::<UserAgent>().unwrap().os.unwrap().version.unwrap().to_string();
        visit.os_name = request.origin.headers.get::<UserAgent>().unwrap().os.unwrap().product.unwrap().to_string();
        visit.is_mobile = request.param("is_mobile").unwrap().to_string().parse::<bool>().unwrap();
        visit.is_bot = request.param("is_bot").unwrap().to_string().parse::<bool>().unwrap();

        conn.execute("INSERT INTO visits (host, page_url, country, anonymous_id, visited_at, window_width, is_pwa_mode, user_agent, browser_version, browser_name, os_version, os_name, is_mobile, is_bot) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
            &[&visit.host, &visit.page_url, &visit.country, &visit.anonymous_id, &visit.visited_at, &visit.window_width, &visit.is_pwa_mode, &visit.user_agent, &visit.browser_version, &visit.browser_name, &visit.os_version, &visit.os_name, &visit.is_mobile, &visit.is_bot]).unwrap();

        return response.send(format!("{:?}", visit));
    });

    api.get("/api/visits", middleware! { |request, response|
        let conn = getDbConnection().unwrap();
        let mut visits = Vec::new();
        for row in &conn.query("SELECT * FROM visits", &[]).unwrap() {
            let visit = Visit {
                id: row.get(0),
                host: row.get(1),
                page_url: row.get(2),
                country: row.get(3),
                anonymous_id: row.get(4),
                visited_at: row.get(5),
                window_width: row.get(6),
                is_pwa_mode: row.get(7),
                user_agent: row.get(8),
                browser_version: row.get(9),
                browser_name: row.get(10),
                os_version: row.get(11),
                os_name: row.get(12),
                is_mobile: row.get(13),
                is_bot: row.get(14)
            };
            visits.push(visit);
        }
        return response.send(format!("{:?}", visits));
    });

    api.get("/api/navigations", middleware! { |request, response|
        let conn = getDbConnection().unwrap();
        let mut navigations = Vec::new();
        for row in &conn.query("SELECT * FROM navigations", &[]).unwrap() {
            let navigation = Navigation {
                id: row.get(0),
                page_url: row.get(1),
                rank: row.get(2),
                time_spent: row.get(3),
                visit_id: row.get(4)
            };
            navigations.push(navigation);
        }
        return response.send(format!("{:?}", navigations));
    });
}

// look up country from ip ranges in db
fn getCountryFromIp(ip: String) -> Result<String, String> {
    let conn = getDbConnection().unwrap();
    let mut country = String::new();
    for row in &conn.query("SELECT country FROM ip_ranges WHERE $1 BETWEEN start_ip AND end_ip", &[&ip]).unwrap() {
        country = row.get(0);
    }
    return Ok(country);
}

fn getAnonymousId(ip: String, user_agent: String) -> String {
    let mut hasher = Sha256::new();
    hasher.input_str(&ip);
    hasher.input_str(&user_agent);
    // add day to hash to make it expire
    hasher.input_str(&Utc::now().format("%Y-%m-%d").to_string());
    return hasher.result_str();
}