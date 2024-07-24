use graph::Graph;

fn make_graph() -> Graph<char, u32> {
    let mut graph = Graph::new();
    let a = graph.insert('A').weak();
    let b = graph.insert('B').weak();
    let c = graph.insert('C').weak();
    graph.connect_undirected_weighted(a, c, 3);
    graph.connect_undirected_weighted(a, b, 1);
    graph.connect_undirected_weighted(b, c, 1);
    graph
}

#[test]
pub fn test_dijkstras() {
    let graph = make_graph();
    let a = graph.find(&'A').unwrap();
    let b = graph.find(&'C').unwrap();

    let path = graph.dijkstras(a, b).unwrap();
    println!("{path:?}");
    assert_eq!(path.len(), 2);

    let mut path_iter = path.into_iter();

    let first = path_iter.next().unwrap();
    assert_eq!(*first, 'A');
    let second = path_iter.next().unwrap();
    assert_eq!(*second, 'B');
    let third = path_iter.next().unwrap();
    assert_eq!(*third, 'C');
    let fourth = path_iter.next();
    assert!(fourth.is_none());
}
