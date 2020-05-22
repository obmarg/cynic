use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut schema = File::open("cms-schema.gql").unwrap();
    let mut schema_data = String::new();
    schema.read_to_string(&mut schema_data).unwrap();

    let query = r#"
    query MenuQuery($id: ID!) {
        menu(id: $id) {
          id
          name
          market
          start_date
          end_date
          bastes {
            baste {
              id
              slug
              name
              description
            }
          }
          menu_section {
            id
            section_name
            section_description
            menuitems(sort: "order:asc") {
              id
              name
              display_name
              currency
              description
              Fulfilment {
                delivery
                collection
                eat_in
              }
              recipe {
                basteable
                addable_ingredients {
                  name
                  id
                  additional_price_gbp
                  additional_price_eur
                }
                removable_ingredients {
                  name
                  id
                  additional_price_gbp
                  additional_price_eur
                }
                nutritional_facts {
                  energy_kcal
                  energy_kj
                  fat_mg
                  fibre_mg
                  protein_mg
                  salt_mg
                  saturates_mg
                  sugars_mg
                  total_carbs_mg
                }
                allergens_group {
                  Celery
                  CeleryMessageOverride
                  Crustacean
                  CrustaceanMessageOverride
                  Egg
                  EggMessageOverride
                  Fish
                  FishMessageOverride
                  Gluten
                  GlutenMessageOverride
                  GlutenBarley
                  GlutenBarleyMessageOverride
                  GlutenOats
                  GlutenOatsMessageOverride
                  GlutenRye
                  GlutenRyeMessageOverride
                  GlutenSpelt
                  GlutenSpeltMessageOverride
                  GlutenWheat
                  GlutenWheatMessageOverride
                  Lupin
                  LupinMessageOverride
                  Milk
                  MilkMessageOverride
                  Mollusc
                  MolluscMessageOverride
                  Mustard
                  MustardMessageOverride
                  Nut
                  NutMessageOverride
                  Peanut
                  PeanutMessageOverride
                  Sesame
                  SesameMessageOverride
                  Soya
                  SoyaMessageOverride
                  Sulphite
                  SulphiteMessageOverride
                }
                diets {
                  diabetic
                  gluten_free
                  halal
                  hindu
                  kosher
                  low_calorie
                  low_fat
                  low_salt
                  low_lactose
                  vegan
                  vegetarian
                }
              }
              modifiers {
                id
                kind
                price
              }
              side_attributes {
                side_type
                price_as_side
              }
            }
          }
        }
      }
    "#;

    println!(
        "{:?}",
        cynic_querygen::document_to_fragment_structs(
            query,
            schema_data,
            &cynic_querygen::QueryGenOptions::default()
        )
        .unwrap()
    );
}
