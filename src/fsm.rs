use std::any::Any;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct StateId(u32);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TransitionId(u32);

pub struct Edge {
    src_id: StateId,
    dest_id: StateId,
}

impl Edge {
    pub fn new(src_id: StateId, dest_id: StateId) -> Edge {
        Edge { src_id, dest_id }
    }
}

pub trait Transition: Deref<Target = Edge> {
    fn satisfied(&self, state: &dyn State) -> bool;
}

pub struct Direct {
    edge: Edge,
}
impl Direct {
    pub fn new(src_id: StateId, dest_id: StateId) -> Direct {
        Direct {
            edge: Edge::new(src_id, dest_id),
        }
    }
}
impl Deref for Direct {
    type Target = Edge;
    fn deref(&self) -> &Self::Target {
        &self.edge
    }
}
impl Transition for Direct {
    fn satisfied(&self, _: &dyn State) -> bool {
        true
    }
}

pub struct Node {
    in_set: HashSet<TransitionId>,
    out_set: HashSet<TransitionId>,
}

impl Node {
    pub fn new() -> Node {
        Node {
            in_set: HashSet::new(),
            out_set: HashSet::new(),
        }
    }

    fn add_in(&mut self, id: TransitionId) {
        self.in_set.insert(id);
    }

    fn remove_in(&mut self, id: TransitionId) {
        self.in_set.remove(&id);
    }

    fn add_out(&mut self, id: TransitionId) {
        self.out_set.insert(id);
    }

    fn remove_out(&mut self, id: TransitionId) {
        self.out_set.remove(&id);
    }
}

pub trait State: Deref<Target = Node> + DerefMut<Target = Node> + Any {
    fn as_any(&self) -> &dyn Any;
    fn enter(&mut self) {}
    fn tick(&mut self) {}
    fn can_exit(&self) -> bool {
        true
    }
    fn exit(&mut self) {}
}

pub struct Entry {
    node: Node,
}
impl Entry {
    fn new() -> Entry {
        Entry { node: Node::new() }
    }
}
impl Deref for Entry {
    type Target = Node;
    fn deref(&self) -> &Self::Target {
        &self.node
    }
}
impl DerefMut for Entry {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}
impl State for Entry {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Exit {
    node: Node,
}
impl Exit {
    fn new() -> Exit {
        Exit { node: Node::new() }
    }
}
impl Deref for Exit {
    type Target = Node;
    fn deref(&self) -> &Self::Target {
        &self.node
    }
}
impl DerefMut for Exit {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.node
    }
}
impl State for Exit {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Fsm {
    state_map: HashMap<StateId, Box<dyn State>>,
    transition_map: HashMap<TransitionId, Box<dyn Transition>>,

    entry_state_id: StateId,
    exit_state_id: StateId,

    exited: bool,

    curr_state_id: StateId,
    state_id_counter: StateId,
    transition_id_counter: TransitionId,
}

impl Fsm {
    pub fn new() -> Fsm {
        let mut fsm = Fsm {
            state_map: HashMap::new(),
            transition_map: HashMap::new(),
            entry_state_id: StateId(0),
            exit_state_id: StateId(0),
            exited: false,
            curr_state_id: StateId(0),
            state_id_counter: StateId(0),
            transition_id_counter: TransitionId(0),
        };
        fsm.entry_state_id = fsm.add_state(Box::new(Entry::new()));
        fsm.exit_state_id = fsm.add_state(Box::new(Exit::new()));
        fsm.curr_state_id = fsm.entry_state_id;
        fsm
    }

    pub fn entry_state_id(&self) -> StateId {
        self.entry_state_id
    }
    pub fn exit_state_id(&self) -> StateId {
        self.exit_state_id
    }
    pub fn curr_state_id(&self) -> StateId {
        self.curr_state_id
    }
    pub fn curr_state(&self) -> &dyn State {
        self.state_map[&self.curr_state_id].as_ref()
    }
    pub fn curr_state_mut(&mut self) -> &mut dyn State {
        self.state_map
            .get_mut(&self.curr_state_id)
            .unwrap()
            .as_mut()
    }

    pub fn add_state(&mut self, state: Box<dyn State>) -> StateId {
        self.state_id_counter.0 += 1;
        self.state_map.insert(self.state_id_counter, state);
        self.state_id_counter
    }

    pub fn remove_state(&mut self, id: StateId) {
        if let Some(state) = self.state_map.remove(&id) {
            for id in &state.in_set {
                self.transition_map.remove(id);
            }
            for id in &state.out_set {
                self.transition_map.remove(id);
            }
        }
    }

    pub fn add_transition(&mut self, transition: Box<dyn Transition>) -> Option<TransitionId> {
        if self.state_map.contains_key(&transition.src_id)
            && self.state_map.contains_key(&transition.dest_id)
        {
            self.transition_id_counter.0 += 1;

            let src_state = self.state_map.get_mut(&transition.src_id).unwrap();
            src_state.add_out(self.transition_id_counter);

            let dest_state = self.state_map.get_mut(&transition.dest_id).unwrap();
            dest_state.add_in(self.transition_id_counter);

            self.transition_map
                .insert(self.transition_id_counter, transition);
            return Some(self.transition_id_counter);
        }
        None
    }

    pub fn remove_transition(&mut self, id: TransitionId) {
        if let Some(transition) = self.transition_map.remove(&id) {
            let src_state = self.state_map.get_mut(&transition.src_id).unwrap();
            src_state.remove_out(id);

            let dest_state = self.state_map.get_mut(&transition.dest_id).unwrap();
            dest_state.remove_in(id);
        }
    }

    pub fn tick(&mut self) {
        if self.exited {
            return;
        }
        self.curr_state_mut().tick();
        let curr_state = self.curr_state();
        if curr_state.can_exit() {
            for transition_id in &curr_state.out_set {
                let transition = &self.transition_map[transition_id];
                if transition.satisfied(curr_state) {
                    let next_state_id = transition.dest_id;
                    let curr_state = self.curr_state_mut();
                    curr_state.exit();
                    self.curr_state_id = next_state_id;
                    let curr_state = self.state_map.get_mut(&self.curr_state_id).unwrap();
                    curr_state.enter();

                    if self.curr_state_id == self.exit_state_id {
                        self.exited = true;
                    }
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Eat {
        node: Node,
        food: String,
        food_sum: u32,
        food_count: u32,
    }
    impl Eat {
        fn new(food: String, food_sum: u32) -> Eat {
            Eat {
                node: Node::new(),
                food,
                food_sum,
                food_count: food_sum,
            }
        }
    }
    impl Deref for Eat {
        type Target = Node;
        fn deref(&self) -> &Self::Target {
            &self.node
        }
    }
    impl DerefMut for Eat {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.node
        }
    }
    impl State for Eat {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn enter(&mut self) {
            self.food_count = self.food_sum;
            self.food_count -= 1;
        }
        fn tick(&mut self) {
            self.food_count -= 1;
        }
    }

    struct Nap {
        node: Node,
        tick_count: u32,
    }
    impl Nap {
        fn new(tick_count: u32) -> Nap {
            Nap {
                node: Node::new(),
                tick_count,
            }
        }
    }
    impl Deref for Nap {
        type Target = Node;
        fn deref(&self) -> &Self::Target {
            &self.node
        }
    }
    impl DerefMut for Nap {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.node
        }
    }
    impl State for Nap {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn tick(&mut self) {
            self.tick_count -= 1;
        }
    }

    struct EatFinished {
        edge: Edge,
        food: String,
    }
    impl EatFinished {
        fn new(src_id: StateId, dest_id: StateId, food: String) -> EatFinished {
            EatFinished {
                edge: Edge::new(src_id, dest_id),
                food,
            }
        }
    }
    impl Deref for EatFinished {
        type Target = Edge;
        fn deref(&self) -> &Self::Target {
            &self.edge
        }
    }
    impl Transition for EatFinished {
        fn satisfied(&self, state: &dyn State) -> bool {
            if let Some(state) = state.as_any().downcast_ref::<Eat>() {
                return state.food_count == 0 && state.food == self.food;
            }
            false
        }
    }

    struct NapOnce {
        edge: Edge,
    }
    impl NapOnce {
        fn new(src_id: StateId, dest_id: StateId) -> NapOnce {
            NapOnce {
                edge: Edge::new(src_id, dest_id),
            }
        }
    }
    impl Deref for NapOnce {
        type Target = Edge;
        fn deref(&self) -> &Self::Target {
            &self.edge
        }
    }
    impl Transition for NapOnce {
        fn satisfied(&self, state: &dyn State) -> bool {
            if let Some(state) = state.as_any().downcast_ref::<Nap>() {
                return state.tick_count > 0;
            }
            false
        }
    }

    struct NapFinished {
        edge: Edge,
    }
    impl NapFinished {
        fn new(src_id: StateId, dest_id: StateId) -> NapFinished {
            NapFinished {
                edge: Edge::new(src_id, dest_id),
            }
        }
    }
    impl Deref for NapFinished {
        type Target = Edge;
        fn deref(&self) -> &Self::Target {
            &self.edge
        }
    }
    impl Transition for NapFinished {
        fn satisfied(&self, state: &dyn State) -> bool {
            if let Some(state) = state.as_any().downcast_ref::<Nap>() {
                return state.tick_count == 0;
            }
            false
        }
    }

    #[test]
    fn exit_directly() {
        let mut fsm = Fsm::new();
        let entry_id = fsm.entry_state_id();
        let exit_id = fsm.exit_state_id();
        fsm.add_transition(Box::new(Direct::new(entry_id, exit_id)));
        assert_eq!(fsm.curr_state_id(), entry_id);
        fsm.tick();
        assert_eq!(fsm.curr_state_id(), exit_id);
        fsm.tick();
        assert_eq!(fsm.curr_state_id(), exit_id);
    }

    #[test]
    fn eat_fish_and_then_bread() {
        let mut fsm = Fsm::new();
        let entry_id = fsm.entry_state_id();
        let exit_id = fsm.exit_state_id();
        let eat_fish_id = fsm.add_state(Box::new(Eat::new("Fish".to_string(), 2)));
        let eat_chip_id = fsm.add_state(Box::new(Eat::new("Chip".to_string(), 3)));

        fsm.add_transition(Box::new(Direct::new(entry_id, eat_fish_id)));
        fsm.add_transition(Box::new(EatFinished::new(
            eat_fish_id,
            eat_chip_id,
            "Fish".to_string(),
        )));
        fsm.add_transition(Box::new(EatFinished::new(
            eat_fish_id,
            exit_id,
            "Chip".to_string(),
        )));
        fsm.add_transition(Box::new(EatFinished::new(
            eat_chip_id,
            exit_id,
            "Chip".to_string(),
        )));

        assert_eq!(fsm.curr_state_id(), entry_id);
        fsm.tick();
        assert_eq!(fsm.curr_state_id(), eat_fish_id);
        fsm.tick();
        assert_eq!(fsm.curr_state_id(), eat_chip_id);
        fsm.tick();
        assert_eq!(fsm.curr_state_id(), eat_chip_id);
        fsm.tick();
        assert_eq!(fsm.curr_state_id(), exit_id);
    }

    #[test]
    fn eat_fish_and_then_nap() {
        let mut fsm = Fsm::new();
        let entry_id = fsm.entry_state_id();
        let exit_id = fsm.exit_state_id();
        let eat_fish_id = fsm.add_state(Box::new(Eat::new("Fish".to_string(), 2)));
        let nap_id = fsm.add_state(Box::new(Nap::new(2)));

        fsm.add_transition(Box::new(Direct::new(entry_id, eat_fish_id)));
        fsm.add_transition(Box::new(EatFinished::new(
            eat_fish_id,
            nap_id,
            "Fish".to_string(),
        )));
        fsm.add_transition(Box::new(EatFinished::new(
            eat_fish_id,
            exit_id,
            "Chip".to_string(),
        )));
        fsm.add_transition(Box::new(NapOnce::new(nap_id, exit_id)));

        assert_eq!(fsm.curr_state_id(), entry_id);
        fsm.tick();
        assert_eq!(fsm.curr_state_id(), eat_fish_id);
        fsm.tick();
        assert_eq!(fsm.curr_state_id(), nap_id);
        fsm.tick();
        assert_eq!(fsm.curr_state_id(), exit_id);
    }

    #[test]
    fn eat_and_nap_by_turn() {
        let mut fsm = Fsm::new();
        let entry_id = fsm.entry_state_id();
        let exit_id = fsm.exit_state_id();
        let eat_fish_id = fsm.add_state(Box::new(Eat::new("Fish".to_string(), 3)));
        let nap_id = fsm.add_state(Box::new(Nap::new(2)));

        fsm.add_transition(Box::new(Direct::new(entry_id, eat_fish_id)));
        fsm.add_transition(Box::new(EatFinished::new(
            eat_fish_id,
            nap_id,
            "Fish".to_string(),
        )));
        fsm.add_transition(Box::new(EatFinished::new(
            eat_fish_id,
            exit_id,
            "Chip".to_string(),
        )));
        fsm.add_transition(Box::new(NapOnce::new(nap_id, eat_fish_id)));
        fsm.add_transition(Box::new(NapFinished::new(nap_id, exit_id)));

        assert_eq!(fsm.curr_state_id(), entry_id);
        fsm.tick();
        assert_eq!(fsm.curr_state_id(), eat_fish_id);
        fsm.tick();
        assert_eq!(fsm.curr_state_id(), eat_fish_id);
        fsm.tick();
        assert_eq!(fsm.curr_state_id(), nap_id);
        fsm.tick();
        assert_eq!(fsm.curr_state_id(), eat_fish_id);
        fsm.tick();
        assert_eq!(fsm.curr_state_id(), eat_fish_id);
        fsm.tick();
        assert_eq!(fsm.curr_state_id(), nap_id);
        fsm.tick();
        assert_eq!(fsm.curr_state_id(), exit_id);
    }
}
