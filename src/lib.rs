use std::collections::{HashMap, HashSet};

use itertools::all;
use serde_json::Value;

fn to_stringpair_opt(value: Value) -> Option<(String, String)> {
    match value {
        Value::Array(vec) => match vec.as_slice() {
            [Value::String(s0), Value::String(s1)] if !s0.is_empty() && !s1.is_empty() => {
                Some((s0.to_string(), s1.to_string()))
            }
            _ => None,
        },
        _ => None,
    }
}

fn to_stringpairs(value: Value) -> Result<Vec<(String, String)>, String> {
    match value {
        Value::Array(vec) if !vec.is_empty() => {
            let opt_pairs: Vec<Option<(String, String)>> =
                vec.into_iter().map(to_stringpair_opt).collect();
            if all(opt_pairs.clone(), |elt: Option<(String, String)>| {
                elt.is_some()
            }) {
                Ok(opt_pairs
                    .into_iter()
                    .map(|elt: Option<(String, String)>| elt.unwrap())
                    .collect())
            } else {
                Err("some invalid entries".to_string())
            }
        }
        _ => Err("value is not an array".to_string()),
    }
}



fn to_js_stringpair(stringpair: (String, String)) -> Value {
    let (s0, s1) = stringpair;
    Value::Array(vec![Value::String(s0), Value::String(s1)])
}

fn to_js_itinerary(vec: Vec<(String, String)>) -> Value {
    Value::Array(vec.into_iter().map(to_js_stringpair).collect())
}

fn itinerary_sort(vec: Vec<(String, String)>) -> Result<Vec<(String, String)>, String> {
    // All we need to do is find the string which is in the set of "from" locations
    // which is not in the set of "to" locations. That allows us to pick the first flight.
    // Once the first flight is picked, the rest of the flights should follow uniquely. If
    // not, there's an error

    let mut res: Vec<(String, String)> = Vec::new();
    let from_locs: HashSet<String> = HashSet::from_iter(vec.iter().map(|elt| elt.0.clone()));
    let to_locs: HashSet<String> = HashSet::from_iter(vec.iter().map(|elt| elt.1.clone()));
    let mut unordered: HashMap<String, String> = HashMap::from_iter(vec.into_iter());

    let mut first_loc: Option<String> = None;

    for from in from_locs.iter() {
        if !to_locs.contains(from) {
            first_loc = Some(from.to_string());
            break;
        }
    }

    if first_loc.is_none() {
        return Err("No first location, possible cycle".to_string());
    }

    let mut start = first_loc.unwrap();

    while !unordered.is_empty() {
        if let Some(elt) = unordered.remove_entry(&start) {
            start = elt.1.clone();
            res.push(elt);
        } else {
            return Err("No first location, possible cycle".to_string());
        }
    }
    Ok(res)
}

pub fn js_itinerary_sort(value: Value) -> Result<Value, String> {
    let pairs = to_stringpairs(value)?;
    let sorted = itinerary_sort(pairs)?;
    Ok(to_js_itinerary(sorted))
}

pub fn js_itinerary_termini(value: Value) -> Result<Value, String> {
    let pairs = to_stringpairs(value)?;
    let sorted = itinerary_sort(pairs)?;
    let termini = vec![(sorted[0].0.clone(), sorted[sorted.len() - 1].1.clone())];
    Ok(to_js_itinerary(termini))
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_eq;

    #[test]
    fn test_single_flight() {
        // Some JSON input data as a &str. Maybe this comes from the user.
        let data0 = r#"
    [["SFO", "EWR"]]
    "#;
        let v0: Value = serde_json::from_str(data0).unwrap();
        let v0s = js_itinerary_sort(v0.clone()).unwrap();
        assert_json_eq!(v0, v0s);
    }
    #[test]
    fn test_double_flight() {
        let data1 = r#"
    [["ATL", "EWR"], ["SFO", "ATL"]] 
    "#;
        let v1: Value = serde_json::from_str(data1).unwrap();
        let v1s = js_itinerary_sort(v1.clone()).unwrap();
        let data1s = r#"
    [["SFO", "ATL"], ["ATL", "EWR"]] 
    "#;
        assert_json_eq!(v1s, serde_json::from_str::<Value>(data1s).unwrap());
    }

    #[test]
    fn test_triple_flight() {
        let data2 = r#"
    [["IND", "EWR"], ["SFO", "ATL"], ["GSO", "IND"], ["ATL", "GSO"]]
        "#;
        let v2: Value = serde_json::from_str(data2).unwrap();
        let v2s = js_itinerary_sort(v2.clone()).unwrap();
        let data2s = r#"
        [["SFO", "ATL"], ["ATL", "GSO"], ["GSO", "IND"], ["IND", "EWR"]]
        "#;
        assert_json_eq!(v2s, serde_json::from_str::<Value>(data2s).unwrap());
    }

    #[test]
    fn test_flight_cycle() {
        let data = r#"
    [["IND", "EWR"], ["SFO", "ATL"], ["GSO", "IND"], ["ATL", "GSO"], ["EWR", "SFO"]]
        "#;
        let v: Value = serde_json::from_str(data).unwrap();
        let vs = js_itinerary_sort(v.clone());
        assert!(vs.is_err());
    }
}
