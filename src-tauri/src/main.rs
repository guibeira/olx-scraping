// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, fs::OpenOptions, io::Write, process::{exit, Command}, thread, time::Duration};
use thirtyfour::{error::WebDriverError, extensions::query::ElementWaitable, By, DesiredCapabilities, WebDriver};
use url::Url;

use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
            let window = app.get_window("main").unwrap();
            window.open_devtools();
            window.close_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn greet(name: String) -> String {
    let url = Url::parse(&name);
    if url.is_err() {
        return "Url inválida".to_string();
    }
    let url = url.unwrap();
    //start_chromedriver().await.expect("Error starting chromedriver");
    let driver = initialize_driver().await.unwrap();
    scrape_olx(driver.clone(), url).await.unwrap();
    driver.quit().await.unwrap();
    format!("Pesquisa finalizada")
}

async fn start_chromedriver() -> Result<(), Box<dyn Error>> {
    let is_chromedriver_installed = Command::new("chromedriver").arg("--version").output().is_ok();
    if !is_chromedriver_installed {
        println!("Chromedriver is not installed");
        exit(1);
    }
    Command::new("chromedriver")
        .arg("--port=9515")
        .spawn()
        .expect("Error starting chromedriver"); 
    thread::sleep(Duration::from_secs(2));
    Ok(())
}

async fn initialize_driver() -> Result<WebDriver, WebDriverError> {
    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:9515", caps).await?;
    driver.maximize_window().await?;
    Ok(driver)
}

async fn car_details(driver: WebDriver) -> Result<(), WebDriverError> {
    // get the car details
    let tabs = driver.windows().await?;
    let second_tab = tabs.get(1).expect("No second tab found");
    // switch to the new tab
    driver.switch_to_window(second_tab.clone()).await?;
    driver
        .execute("window.scrollTo(0, document.body.scrollHeight)", vec![])
        .await?;
    thread::sleep(Duration::from_secs(1));
    //scrool to the top of the page
    driver.execute("window.scrollTo(0, 0)", vec![]).await?;

    // let title = driver.find(By::Css("h1")).await?;
    // let title = title.text().await?;
    // fipe price
    // #content > div.ad__sc-18p038x-2.djeeke > div > div.sc-bwzfXH.ad__sc-h3us20-0.lbubah > div.ad__sc-duvuxf-0.ad__sc-h3us20-0.hRTDUb > div.ad__sc-h3us20-6.fnDpgM > div > div > div > div.sc-bcXHqe.sc-eDvSVe.caEdXs.hKQPaV > div:nth-child(2) > div > span
    let fipe_price = driver.find(By::Css("#content > div.ad__sc-18p038x-2.djeeke > div > div.sc-bwzfXH.ad__sc-h3us20-0.lbubah > div.ad__sc-duvuxf-0.ad__sc-h3us20-0.hRTDUb > div.ad__sc-h3us20-6.fnDpgM > div > div > div > div.sc-bcXHqe.sc-eDvSVe.caEdXs.hKQPaV > div:nth-child(2) > div > span")).await;
    if fipe_price.is_err() {
        println!("No fipe price found");
        return Ok(());
    }
    let fipe_price = fipe_price.unwrap();
    let fipe_price = fipe_price.text().await.unwrap();
    // price
    // #content > div.ad__sc-18p038x-2.djeeke > div > div.sc-bwzfXH.ad__sc-h3us20-0.lbubah > div.ad__sc-duvuxf-0.ad__sc-h3us20-0.hRTDUb > div.ad__sc-h3us20-6.fnDpgM > div > div > div > div.sc-bcXHqe.iWpWFh > div > div > div.sc-bcXHqe.caEdXs > div > div > span.sc-gswNZR.bwsIJy
    let price = driver.find(By::Css("#content > div.ad__sc-18p038x-2.djeeke > div > div.sc-bwzfXH.ad__sc-h3us20-0.lbubah > div.ad__sc-duvuxf-0.ad__sc-h3us20-0.dBnjzp > div.ad__sc-h3us20-6.kwBCR > div > div > div.olx-d-flex.olx-ai-flex-start.olx-fd-row > div > h2:nth-child(2)")).await;
    if price.is_err() {
        println!("No price found");
        return Ok(());
    }
    let price = price.unwrap();
    let price = price.text().await.expect("No price found");
    let fipe_price = fipe_price.replace("R$ ", "").replace(".", "").replace(",", ".").parse::<f64>().expect("Error parsing price");
    let price = price.replace("R$ ", "").replace(".", "").replace(",", ".").parse::<f64>().expect("Error parsing price");
    // println!("Price: {}", price);
    // println!("Fipe: {}", fipe_price);
    // println!("Title: {}", title);
    if price < fipe_price - 10_000.0 {
        //println!("Price is below fipe");
        // print link
        let link = driver.current_url().await?;
        println!("Link: {}", link);
        // save the link in a file prices.txt, create the file if it does not exist
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open("prices.txt")?;
        file.write_all(format!("{}\n", link).as_bytes())?;

    } 
    // close the tab
    Ok(())
}

async fn navegate_cars(driver: WebDriver) -> Result<(), WebDriverError>{
    // .sc-a8d048d5-2.fGmfQo a.olx-ad-card__link-wrapper'
    let car_list = driver
        .find_all(By::Css(".sc-a8d048d5-2.fGmfQo a.olx-ad-card__link-wrapper"))
        .await;
    if car_list.is_err() {
        println!("No cars found");
        driver.quit().await?;
        return Ok(());
    }
    let car_list = car_list.unwrap();
    println!("Found {} cars", car_list.len());
    let car_len = car_list.len();
    let mut count = 0;
    let mut scroll_multiplier = 1;
    while count < car_len {
        //click on the car
        let tabs = driver.windows().await?;
        let first_tab = tabs.get(0).expect("No main tab");
        driver.switch_to_window(first_tab.clone()).await?;
        thread::sleep(Duration::from_secs(1));
        //scroll to the car
        // check if count is multiple of 3
        let should_scroll = count % 3 == 0;
        if should_scroll {
            let position = scroll_multiplier * 380;
            let position = format!("window.scrollTo(0, {})", position);
            println!("Scrolling to the car :{}", position);
            driver
                .execute(position, vec![])
                .await
                .expect("Error scrolling to the car");
            thread::sleep(Duration::from_secs(2));
            scroll_multiplier += 1;
        }
        println!("Clicking on the car :{}", count);
        let car = car_list.get(count).expect("No car found");
        //position the car in the middle of the screen
        car.wait_until()
            .clickable()
            .await
            .expect("Error waiting for the car to be clickable");
        car.click().await.expect("Error clicking on the car");
        thread::sleep(Duration::from_secs(2));
        car_details(driver.clone()).await?;
        let tabs = driver.windows().await?;
        let second_tab = tabs.get(1).expect("No second tab found");
        // switch to the new tab
        driver.switch_to_window(second_tab.clone()).await?;
        // get the car details
        // let title = driver.find(By::Css("h1")).await?;
        // let title = title.text().await?;
        // println!("Title: {}", title);
        // close the tab
        driver.close_window().await?;
        count += 1;
    }
    Ok(())
}

pub async fn scrape_olx(driver: WebDriver, url: Url) -> Result<(), Box<dyn Error>> {
    driver.goto(url).await?;
    thread::sleep(Duration::from_secs(2));
    loop {
        driver .execute("window.scrollTo(0, document.body.scrollHeight)", vec![]) .await?;
        thread::sleep(Duration::from_secs(1));
        //scrool to the top of the page
        driver.execute("window.scrollTo(0, 400)", vec![]).await?;
        thread::sleep(Duration::from_secs(1));
        println!("Starting scrap page");
        navegate_cars(driver.clone()).await?;
        let tabs = driver.windows().await?;
        let first_tab = tabs.get(0).expect("No main tab");
        driver.switch_to_window(first_tab.clone()).await?;
        let next_page_button = driver.find(By::Css("#listing-pagination > aside > div > a:nth-child(12)")).await;
        if next_page_button.is_err() {
            println!("No more pages to scrape");
            break;
        }
        let next_page_button = next_page_button.unwrap();
        next_page_button.click().await?;
    }
    // close the browser
    driver.quit().await?;
    Ok(())
}
