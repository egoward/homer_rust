pub struct Metric {
    pub name: String,
    pub value: String,
}

pub trait MetricDestination {
    fn name(&self) -> String;
    fn report(&self, metrics: &Vec<Metric>) -> ();
}

pub struct MetricDestinationLog {}

impl MetricDestination for MetricDestinationLog {
    fn name(&self) -> String {
        return String::from("MetricDestinationLog");
    }
    fn report(&self, metrics: &Vec<Metric>) {
        for metric in metrics {
            println!("Metric {} has value {}", metric.name, metric.value);
        }
    }
}

pub trait MetricSource {
    fn name(&self) -> String;
    fn poll(&self) -> Vec<Metric>;
}

pub struct MetricSourceTest {}

impl MetricSource for MetricSourceTest {
    fn name(&self) -> String {
        return String::from("MetricSourceTest");
    }
    fn poll(&self) -> Vec<Metric> {
        return vec![Metric {
            name: String::from("TestMetric"),
            value: String::from("1.0"),
        }];
    }
}


pub struct MetricManager {
    pub destinations : Vec<Box<dyn MetricDestination>>,
    pub sources : Vec<Box<dyn MetricSource>>,
}

impl MetricManager {
    pub fn run(&self) {
        println!("Running metric manager");
        for source in &self.sources {
            println!("Checking {}", source.name());
            let metrics = source.poll();
            for destination in &self.destinations {
                println!("Sending to {}", destination.name());
                destination.report( &metrics );
            }
        }
    }
}