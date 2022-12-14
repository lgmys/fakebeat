use std::collections::HashMap;

use fake::Fake;
use rand::{distributions::Alphanumeric, seq::SliceRandom, thread_rng, Rng};

use chrono::{Duration, Utc};

use serde_json::{from_value, to_value};
use tera::{Context, Function, Result, Tera, Value};

const FORMAT_ISO: &str = "%FT%T%z";

pub struct DocumentRenderer {
    generators: HashMap<String, String>,
    tera: Tera,
}

impl DocumentRenderer {
    pub fn render(&mut self, template: &str) -> anyhow::Result<String> {
        let context = Context::default();

        match self.tera.render_str(template, &context) {
            Ok(document_string) => Ok(document_string),
            Err(err) => Err(anyhow::anyhow!(err)),
        }
    }

    pub fn get_generators(&self) -> HashMap<String, String> {
        self.generators.clone()
    }

    fn register_generator<F: Function + 'static>(&mut self, name: &str, desc: &str, function: F) {
        self.tera.register_function(name, function);
        self.generators.insert(name.to_owned(), desc.to_owned());
    }

    fn register_generators(&mut self) {
        self.register_generator(
            "date",
            "Generates random date. Optional negative offset can be passed to specify the amount of days to be subtracted, via 'sub_rnd_days' param.",
            Box::new(move |args: &HashMap<String, Value>| -> Result<Value> {
                match args.get("sub_rnd_days") {
                    Some(sub_rnd_days) => match from_value::<i64>(sub_rnd_days.clone()) {
                        Ok(sub_rnd_days) => {
                            let mut rng = rand::thread_rng();

                            let random_offset = rng.gen_range(0..sub_rnd_days);
                            let dt = Utc::now() - Duration::days(random_offset);

                            Ok(to_value(dt.format(FORMAT_ISO).to_string()).unwrap())
                        }
                        Err(_) => Err("".into()),
                    },
                    None => {
                        let now = Utc::now().format(FORMAT_ISO);
                        Ok(to_value(now.to_string()).unwrap())
                    }
                }
            }),
        );

        self.register_generator(
            "now",
            "Current date",
            Box::new(move |_: &HashMap<String, Value>| -> Result<Value> {
                let now = Utc::now().format(FORMAT_ISO);
                Ok(to_value(now.to_string()).unwrap_or_default())
            }),
        );

        self.register_generator(
            "hash",
            "16-character long alpha-num hash",
            Box::new(move |_: &HashMap<String, Value>| -> Result<Value> {
                let value = thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(16)
                    .map(char::from)
                    .collect::<String>();

                let value = to_value(value).unwrap_or_default();

                Ok(value)
            }),
        );

        self.register_generator(
            "random_value",
            "Get random value from set configured with the 'options' parameter, eg. options='a|b|c'",
            move |args: &HashMap<String, Value>| -> Result<Value> {
                let mut rng = thread_rng();

                match args.get("options") {
                    Some(value) => {
                        if let Some(options) = value.as_str() {
                            let options: Vec<&str> = options.split("|").collect();
                            let random_value = options.choose(&mut rng);

                            if let Some(random_value) = random_value {
                                Ok(random_value.to_owned().into())
                            } else {
                                Ok("".into())
                            }
                        } else {
                            Ok("".into())
                        }
                    }
                    None => Ok("".into()),
                }
            },
        );

        self.register_generator(
            "chance",
            "Roll a dice within range, if 0 is rolled then return first option, else 2nd",
            move |args: &HashMap<String, Value>| -> Result<Value> {
                let mut rng = thread_rng();
                let range = args.get("range").unwrap().as_u64().unwrap();
                let options = args.get("options").unwrap().as_str().unwrap();
                let chance = rng.gen_range(0..range);
                let options: Vec<&str> = options.split("|").collect();

                if chance == 0 {
                    Ok(options.get(0).unwrap().to_owned().into())
                } else {
                    Ok(options.get(1).unwrap().to_owned().into())
                }
            },
        );
        self.register_generator(
            "randomint",
            "Roll a dice within a range",
            move |args: &HashMap<String, Value>| -> Result<Value> {
                let mut rng = thread_rng();
                let range = args
                    .get("range")
                    .unwrap_or(&Value::String("0".to_owned()))
                    .as_u64()
                    .unwrap();
                let range = rng.gen_range(0..range);

                Ok(range.to_string().into())
            },
        );

        macro_rules! register_faker_generators {
            (    $($i:ident: $p:path), *) => {
                    $(
                        self.register_generator(stringify!($i), stringify!($p), Box::new(move |_: &HashMap<String, Value>| -> Result<Value> {
                            let value = to_value($p().fake::<String>()).unwrap_or_default();
                            Ok(value)
                        }));
                    )*
                }
            }

        register_faker_generators!(
            // Numbers
            digit: fake::faker::number::en::Digit,
            // Internet
            username: fake::faker::internet::en::Username,
            domainsuffix: fake::faker::internet::en::DomainSuffix,
            ipv4: fake::faker::internet::en::IPv4,
            ipv6: fake::faker::internet::en::IPv6,
            ip: fake::faker::internet::en::IP,
            macaddress: fake::faker::internet::en::MACAddress,
            freeemail: fake::faker::internet::en::FreeEmail,
            safeemail: fake::faker::internet::en::SafeEmail,
            freeemailprovider: fake::faker::internet::en::FreeEmailProvider,
            // Lorem ipsum
            word: fake::faker::lorem::en::Word,
            // Name
            firstname: fake::faker::name::en::FirstName,
            lastname: fake::faker::name::en::LastName,
            title: fake::faker::name::en::Title,
            suffix: fake::faker::name::en::Suffix,
            name: fake::faker::name::en::Name,
            namewithtitle: fake::faker::name::en::NameWithTitle,
            //Filesystem
            filepath: fake::faker::filesystem::en::FilePath,
            filename: fake::faker::filesystem::en::FileName,
            fileextension: fake::faker::filesystem::en::FileExtension,
            dirpath: fake::faker::filesystem::en::DirPath,
            // Company
            companysuffix: fake::faker::company::en::CompanySuffix,
            companyname: fake::faker::company::en::CompanyName,
            buzzword: fake::faker::company::en::Buzzword,
            buzzwordmiddle: fake::faker::company::en::BuzzwordMiddle,
            buzzwordtail: fake::faker::company::en::BuzzwordTail,
            catchphase: fake::faker::company::en::CatchPhase,
            bsverb: fake::faker::company::en::BsVerb,
            bsadj: fake::faker::company::en::BsAdj,
            bsnoun: fake::faker::company::en::BsNoun,
            bs: fake::faker::company::en::Bs,
            profession: fake::faker::company::en::Profession,
            industry: fake::faker::company::en::Industry,
            // Address
            cityprefix: fake::faker::address::en::CityPrefix,
            citysuffix: fake::faker::address::en::CitySuffix,
            cityname: fake::faker::address::en::CityName,
            countryname: fake::faker::address::en::CountryName,
            countrycode: fake::faker::address::en::CountryCode,
            streetsuffix: fake::faker::address::en::StreetSuffix,
            streetname: fake::faker::address::en::StreetName,
            timezone: fake::faker::address::en::TimeZone,
            statename: fake::faker::address::en::StateName,
            stateabbr: fake::faker::address::en::StateAbbr,
            secondaryaddresstype: fake::faker::address::en::SecondaryAddressType,
            secondaryaddress: fake::faker::address::en::SecondaryAddress,
            zipcode: fake::faker::address::en::ZipCode,
            postcode: fake::faker::address::en::PostCode,
            buildingnumber: fake::faker::address::en::BuildingNumber,
            latitude: fake::faker::address::en::Latitude,
            longitude: fake::faker::address::en::Longitude
        );
    }

    fn new() -> Self {
        let tera = Tera::default();

        let generators = HashMap::<String, String>::new();

        return Self { tera, generators };
    }
}

pub struct DocumentRendererFactory {}

impl DocumentRendererFactory {
    pub fn create_renderer() -> DocumentRenderer {
        let mut document_renderer = DocumentRenderer::new();

        document_renderer.register_generators();

        document_renderer
    }
}

#[cfg(test)]
mod tests {
    const FORMAT_ISO: &str = "%FT%T%z";

    use chrono::Utc;

    use crate::document_renderer::DocumentRendererFactory;

    #[test]
    fn it_replaces_the_generators_with_values() {
        let mut renderer = DocumentRendererFactory::create_renderer();

        let result = renderer
            .render(
                r#"{
            "values": {
              "@timestamp": "{{date()}}",
              "file.hash.md5": "{{hash()}}"
            },
            "index": {
              "mappings": {
                "properties": {
                  "file.hash.md5": { "type": "keyword" },
                  "@timestamp": { "type": "date" }
                }
              }
            }
          }
        "#,
            )
            .unwrap();

        assert_eq!(result.contains("date()"), false);
        assert_eq!(result.contains("hash()"), false);

        let dt = Utc::now();

        assert_eq!(result.contains(&dt.format(FORMAT_ISO).to_string()), true);
    }
}
