use itertools::Itertools;
use ndarray::{ArrayD, IxDyn};
use stdweb::{
    traits::*,
    unstable::TryInto,
    web::{
        document,
        event::{ClickEvent, InputEvent, KeyPressEvent},
        html_element::{InputElement, SelectElement, TextAreaElement},
    },
};

use crate::ui::*;
use crate::State;

pub fn add_node(state: &State) {
    let node_id = state.borrow_mut().insert_node();
    state
        .borrow_mut()
        .set_label(node_id, format!("Node #{}", node_id));

    NodeList::get().add_node(node_id, state, None);

    crate::draw::redraw_graph(&state);
    // switch the editor to the newly created node
    node_edit_tab(state, node_id);
}

pub fn node_edit_tab(state: &State, id: usize) {
    crate::utils::set_selected_button(&format!(".node-{}", id));
    let panel = Panel::get();
    panel.clear();

    // get the current node
    let graph = state.borrow();
    let node = graph.get(id).unwrap();

    // First, the node label
    let div = document().create_element("div").unwrap();
    let label: InputElement = document()
        .create_element("input")
        .unwrap()
        .try_into()
        .unwrap();
    label.set_attribute("size", "16");
    label.set_raw_value(&node.label);
    label.add_event_listener(enclose!((state, id, label) move |event: InputEvent| {
        state.borrow_mut().set_label(id, label.raw_value());
        let node_link = document().query_selector(&format!(".node-{}", id)).unwrap().unwrap();
        crate::utils::clear_children(&node_link);
        node_link.append_child(&document().create_text_node(&label.raw_value()));
        crate::draw::redraw_graph(&state);
    }));
    div.append_child(&document().create_text_node("Node name:"));
    div.append_child(&label);
    panel.element().append_child(&div);

    // Second, a list of values
    let value_list = document().create_element("ul").unwrap();
    value_list.class_list().add("blocky").unwrap();
    value_list.class_list().add("vlist").unwrap();
    let li = document().create_element("li").unwrap();
    li.append_child(&document().create_text_node("Node values:"));
    value_list.append_child(&li);
    for (i, v) in node.values.iter().enumerate() {
        let li = document().create_element("li").unwrap();
        li.append_child(&document().create_text_node(v));
        let a = document().create_element("a").unwrap();
        a.append_child(&document().create_text_node("×"));
        a.set_attribute("href", "#").unwrap();
        a.add_event_listener(enclose!((state, id, i) move |_: ClickEvent| {
            state.borrow_mut().remove_value(id, i);
            node_edit_tab(&state, id);
            crate::draw::redraw_graph(&state);
        }));
        li.append_child(&a);
        value_list.append_child(&li);
    }
    let li = document().create_element("li").unwrap();
    let new_value: InputElement = document()
        .create_element("input")
        .unwrap()
        .try_into()
        .unwrap();
    new_value
        .set_attribute("placeholder", "Add a value")
        .unwrap();
    new_value.set_attribute("size", "16").unwrap();
    new_value.add_event_listener(
        enclose!((state, id, new_value) move |event: KeyPressEvent| {
            if event.key() == "Enter" {
                state.borrow_mut().add_value(id, new_value.raw_value());
                node_edit_tab(&state, id);
                crate::draw::redraw_graph(&state);
            }
        }),
    );
    li.append_child(&new_value);
    value_list.append_child(&li);
    panel.element().append_child(&value_list);

    // Third, a list of parents
    let parents_list = document().create_element("ul").unwrap();
    parents_list.class_list().add("blocky").unwrap();
    parents_list.class_list().add("vlist").unwrap();
    let li = document().create_element("li").unwrap();
    li.append_child(&document().create_text_node("Node parents:"));
    parents_list.append_child(&li);
    for &p in &node.parents {
        let li = document().create_element("li").unwrap();
        let parent = graph.get(p).unwrap();
        li.append_child(&document().create_text_node(&parent.label));
        let a = document().create_element("a").unwrap();
        a.append_child(&document().create_text_node("×"));
        a.set_attribute("href", "#").unwrap();
        a.add_event_listener(enclose!((state, id, p) move |_: ClickEvent| {
            state.borrow_mut().remove_edge(id, p);
            node_edit_tab(&state, id);
            crate::draw::redraw_graph(&state);
        }));
        li.append_child(&a);
        parents_list.append_child(&li);
    }
    let li = document().create_element("li").unwrap();
    let new_parent: SelectElement = document()
        .create_element("select")
        .unwrap()
        .try_into()
        .unwrap();
    let empty_option = document().create_element("option").unwrap();
    empty_option.set_attribute("disabled", "").unwrap();
    empty_option.set_attribute("selected", "").unwrap();
    empty_option.set_attribute("value", "").unwrap();
    empty_option.append_child(&document().create_text_node("-- Add parent --"));
    new_parent.append_child(&empty_option);
    for (i, potential) in graph.iter_nodes() {
        if graph.check_edge_addition(id, i).is_ok() {
            let option = document().create_element("option").unwrap();
            option.set_attribute("value", &format!("{}", i)).unwrap();
            option.append_child(&document().create_text_node(&potential.label));
            new_parent.append_child(&option);
        }
    }
    new_parent.add_event_listener(enclose!((state, id, new_parent) move |event: InputEvent| {
        let parent_id: usize = new_parent.raw_value().parse().unwrap();
        state.borrow_mut().add_edge(id, parent_id).unwrap();
        node_edit_tab(&state, id);
        crate::draw::redraw_graph(&state);
    }));
    li.append_child(&new_parent);
    parents_list.append_child(&li);
    panel.element().append_child(&parents_list);

    // Fourth, the node description
    let div = document().create_element("div").unwrap();
    let description: TextAreaElement = document()
        .create_element("textarea")
        .unwrap()
        .try_into()
        .unwrap();
    description.set_attribute("cols", "40");
    description.set_attribute("rows", "4");
    description.set_attribute("placeholder", "Write a description of your node...");
    description.set_value(&node.description);
    description.add_event_listener(enclose!((state, id, description) move |event: InputEvent| {
        state.borrow_mut().set_description(id, description.value());
    }));
    div.append_child(&description);
    panel.element().append_child(&div);
    panel
        .element()
        .append_child(&document().create_element("hr").unwrap());

    // Fifth, the credencies
    let table = document().create_element("table").unwrap();
    let tr = document().create_element("tr").unwrap();
    let th = document().create_element("th").unwrap();
    if !node.parents.is_empty() {
        th.append_child(&document().create_text_node("Parent values"));
    }
    tr.append_child(&th);
    for v in &node.values {
        let th = document().create_element("th").unwrap();
        th.append_child(&document().create_text_node(&format!("C({})", v)));
        tr.append_child(&th);
    }
    let th = document().create_element("th").unwrap();
    th.append_child(&document().create_text_node("Explanation"));
    tr.append_child(&th);
    table.append_child(&tr);
    if node.parents.is_empty() {
        let tr = document().create_element("tr").unwrap();
        let th = document().create_element("th").unwrap();
        th.append_child(&document().create_text_node("Prior"));
        tr.append_child(&th);
        for i in 0..node.values.len() {
            let td = document().create_element("td").unwrap();
            let input: InputElement = document()
                .create_element("input")
                .unwrap()
                .try_into()
                .unwrap();
            input.set_attribute("name", &format!("prior_{}", i));
            input.set_attribute("size", "4");
            input.set_raw_value(&format!(
                "{}",
                node.credencies
                    .as_ref()
                    .map(|array| array[i])
                    .unwrap_or(0.0)
            ));
            td.append_child(&input);
            tr.append_child(&td);
        }
        let td = document().create_element("td").unwrap();
        let description: TextAreaElement = document()
            .create_element("textarea")
            .unwrap()
            .try_into()
            .unwrap();
        description.set_attribute("cols", "20");
        description.set_attribute("rows", "2");
        description.set_attribute("name", "prior_description");
        description.set_attribute("placeholder", "Description for this row...");
        description.set_value(node.cred_description.get(0).map(|s| &s[..]).unwrap_or(""));
        td.append_child(&description);
        tr.append_child(&td);
        table.append_child(&tr);
    } else {
        // one line in the table for all possible parent values !
        let mut values_iterator = node.parents.iter().map(|&p| {
            let pnode = graph.get(p).unwrap();
            pnode
                .values
                .iter()
                .enumerate()
                .map(move |(i, v)| (p, &pnode.label, i, v))
        });
        for (iv, values) in values_iterator.multi_cartesian_product().enumerate() {
            let tr = document().create_element("tr").unwrap();
            let th = document().create_element("th").unwrap();
            let mut header_texts = values.iter().map(|&(_, p, _, v)| format!("{} = {}", p, v));
            th.append_child(&document().create_text_node(&header_texts.next().unwrap()));
            for txt in header_texts {
                th.append_child(&document().create_element("br").unwrap());
                th.append_child(&document().create_text_node(&txt));
            }
            tr.append_child(&th);
            let label = values
                .iter()
                .map(|&(p, _, v, _)| format!("{}-{}", p, v))
                .join("_");
            for i in 0..node.values.len() {
                let td = document().create_element("td").unwrap();
                let input: InputElement = document()
                    .create_element("input")
                    .unwrap()
                    .try_into()
                    .unwrap();
                input.set_attribute("name", &format!("{}_{}", label, i));
                input.set_attribute("size", "4");
                let mut idx = vec![i];
                idx.extend(values.iter().map(|&(_, _, v, _)| v));
                input.set_raw_value(&format!(
                    "{}",
                    node.credencies
                        .as_ref()
                        .map(|array| array[IxDyn(&idx)])
                        .unwrap_or(0.0)
                ));
                td.append_child(&input);
                tr.append_child(&td);
            }
            let td = document().create_element("td").unwrap();
            let description: TextAreaElement = document()
                .create_element("textarea")
                .unwrap()
                .try_into()
                .unwrap();
            description.set_attribute("cols", "20");
            description.set_attribute("rows", "2");
            description.set_attribute("name", &format!("{}_description", label));
            description.set_attribute("placeholder", "Description for this row...");
            description.set_value(node.cred_description.get(iv).map(|s| &s[..]).unwrap_or(""));
            td.append_child(&description);
            tr.append_child(&td);
            table.append_child(&tr);
        }
    }
    let save_btn = document().create_element("a").unwrap();
    save_btn.append_child(&document().create_text_node("Save credencies"));
    save_btn.set_attribute("href", "#").unwrap();
    save_btn.add_event_listener(enclose!((state, id) move |_: ClickEvent| {
        let (credencies, descriptions) = {
            // get the current node
            let graph = state.borrow();
            let node = graph.get(id).unwrap();
            if node.values.is_empty() {
                console!(log, "Empty values!");
                return;
            }
            // save the credencies into an array
            let mut shape = vec![node.values.len()];
            shape.extend(node.parents.iter().map(|&p| {
                graph.get(p).unwrap().values.len()
            }));
            let count = shape.iter().fold(1, |a, b| a * b);
            let mut credencies = ArrayD::from_shape_vec(IxDyn(&shape), vec![0.0; count]).unwrap();
            let mut descriptions = Vec::new();
            if node.parents.is_empty() {
                for i in 0..node.values.len() {
                    // get the credencies
                    let query = format!("input[name=\"prior_{}\"]", i);
                    let input = document().query_selector(&query).unwrap().unwrap();
                    let input: InputElement = input.try_into().unwrap();
                    let val = input.raw_value().parse::<f32>().unwrap_or(0.0);
                    credencies[i] = val;
                }
                // get the description for the row
                let query = "textarea[name=\"prior_description\"]";
                let texta = document().query_selector(query).unwrap().unwrap();
                let texta: TextAreaElement = texta.try_into().unwrap();
                descriptions.push(texta.value());
            } else {
                let mut parent_values = node.parents.iter().map(|&p| {
                    let pnode = graph.get(p).unwrap();
                    (0..pnode.values.len()).map(move |i| (p, i))
                });
                for values in parent_values.multi_cartesian_product() {
                    let label = values.iter().map(|&(p, v)| format!("{}-{}", p, v)).join("_");
                    for i in 0..node.values.len() {
                        // get the credencies
                        let query = format!("input[name=\"{}_{}\"]", label, i);
                        let input = document().query_selector(&query).unwrap().unwrap();
                        let input: InputElement = input.try_into().unwrap();
                        let val = input.raw_value().parse::<f32>().unwrap_or(0.0);
                        let mut idx = vec![i];
                        idx.extend(values.iter().map(|(p,v)| v));
                        credencies[IxDyn(&idx)] = val;
                    }
                    // get the description for the row
                    let query = format!("textarea[name=\"{}_description\"]", label);
                    let texta = document().query_selector(&query).unwrap().unwrap();
                    let texta: TextAreaElement = texta.try_into().unwrap();
                    descriptions.push(texta.value());
                }
            }
            (credencies, descriptions)
        };
        state.borrow_mut().set_credencies(id, credencies).unwrap();
        state.borrow_mut().set_cred_descriptions(id, descriptions).unwrap();
    }));

    panel.element().append_child(&table);
    panel.element().append_child(&save_btn);
}

pub fn set_evidence_tab(state: &State) {
    crate::utils::set_selected_button("#btn-observations");
    let panel = Panel::get();
    panel.clear();
    // setup a list of observations for each node:
    let p = document().create_element("p").unwrap();
    p.append_child(&document().create_text_node("Observations for nodes:"));
    panel.element().append_child(&p);

    let graph = state.borrow();

    let ul = document().create_element("ul").unwrap();
    ul.class_list().add("silentlist").unwrap();
    for (id, node) in graph.iter_nodes() {
        let li = document().create_element("li").unwrap();
        li.append_child(&document().create_text_node(&format!("Node \"{}\":", node.label)));
        let observation: SelectElement = document()
            .create_element("select")
            .unwrap()
            .try_into()
            .unwrap();
        // entry to disable observation
        let no_obs = document().create_element("option").unwrap();
        if node.evidence.is_none() {
            no_obs.set_attribute("selected", "").unwrap();
        }
        no_obs.set_attribute("value", "").unwrap();
        observation.append_child(&no_obs);
        // all possible observations
        for (i, v) in node.values.iter().enumerate() {
            let obs = document().create_element("option").unwrap();
            if node.evidence == Some(i) {
                obs.set_attribute("selected", "").unwrap();
            }
            obs.set_attribute("value", &format!("{}", i));
            obs.append_child(&document().create_text_node(v));
            observation.append_child(&obs);
        }
        observation.add_event_listener(
            enclose!((state, id, observation) move |event: InputEvent| {
                let obs_value = observation.raw_value().parse::<usize>().ok();
                state.borrow_mut().set_evidence(id, obs_value);
                crate::draw::redraw_graph(&state);
            }),
        );
        li.append_child(&observation);
        ul.append_child(&li);
    }

    panel.element().append_child(&ul);
}

pub fn compute_evidences(state: &State) {
    crate::utils::set_selected_button("#btn-compute");
    let panel = Panel::get();
    panel.clear();
    // Compute the evidence
    let (mut bayesnet, mapping) = match state.borrow().make_bayesnet() {
        Ok(v) => v,
        Err(()) => {
            let p = document().create_element("p").unwrap();
            p.append_child(&document().create_text_node(
                "Inference cannot be performed if a node has 0 possible values.",
            ));
            panel.element().append_child(&p);
            return;
        }
    };

    for _ in 0..100 {
        bayesnet.step();
    }
    let mut beliefs = bayesnet.beliefs();

    for b in &mut beliefs {
        b.renormalize();
    }

    // Display the output
    let log10 = 10f32.ln();

    let p = document().create_element("p").unwrap();
    p.append_child(&document().create_text_node("Results of the inference:"));
    panel.element().append_child(&p);
    let ul = document().create_element("ul").unwrap();
    ul.class_list().add("silentlist").unwrap();
    ul.class_list().add("widelist").unwrap();
    for (i, mut credencies) in beliefs.iter().enumerate() {
        let li = document().create_element("li").unwrap();

        let state = state.borrow();
        let node = state.get(mapping[i]).unwrap();

        if let Some(obs) = node.evidence {
            li.append_child(&document().create_text_node(&format!(
                "Node \"{}\" is observed to be: \"{}\"",
                node.label, node.values[obs]
            )));
        } else {
            let inner_ul = document().create_element("ul").unwrap();
            inner_ul.class_list().add("posterior").unwrap();

            for (name, belief) in node
                .values
                .iter()
                .zip(credencies.log_probabilities().iter())
            {
                let inner_li = document().create_element("li").unwrap();
                inner_li.append_child(&document().create_text_node(&format!(
                    "{}: {:.2}",
                    name,
                    belief / log10
                )));
                inner_ul.append_child(&inner_li);
            }

            let p = document().create_element("p").unwrap();
            li.append_child(
                &document().create_text_node(&format!("Beliefs for node \"{}\": ", node.label)),
            );
            li.append_child(&inner_ul);
        }
        ul.append_child(&li);
    }

    panel.element().append_child(&ul);

    // Redraw the graph
    crate::draw::draw_computed_graph(state, &mapping, &beliefs);
}
