use std::collections::HashMap;
use std::collections::HashSet;
use std::marker::PhantomData;

mod preset;

pub use preset::*;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct StateId(u32);

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct TransitionId(u32);

pub trait State<C> {
    fn enter(&mut self, ctx: &mut C);
    fn tick(&mut self, ctx: &mut C) -> bool;
    fn exit(&mut self, ctx: &mut C);
}

pub struct Node<C, S: State<C>> {
    _ctx_type: PhantomData<C>,
    in_set: HashSet<TransitionId>,
    out_set: HashSet<TransitionId>,
    state: S,
}

impl<C, S: State<C>> Node<C, S> {
    pub fn new(state: S) -> Node<C, S> {
        Node {
            _ctx_type: PhantomData,
            in_set: Default::default(),
            out_set: Default::default(),
            state,
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

pub trait Transition<C, S: State<C>> {
    fn satisfied(&self, ctx: &mut C, src: &S, dst: &S) -> bool;
}

pub struct Edge<C, S: State<C>, T: Transition<C, S>> {
    _ctx_type: PhantomData<C>,
    _state_type: PhantomData<S>,
    src_id: StateId,
    dst_id: StateId,
    transition: T,
}

impl<C, S: State<C>, T: Transition<C, S>> Edge<C, S, T> {
    pub fn new(src_id: StateId, dst_id: StateId, transition: T) -> Edge<C, S, T> {
        Edge {
            _ctx_type: PhantomData,
            _state_type: PhantomData,
            src_id,
            dst_id,
            transition,
        }
    }
}

pub struct Fsm<C, S: State<C>, T: Transition<C, S>> {
    _ctx_type: PhantomData<C>,

    node_map: HashMap<StateId, Node<C, S>>,
    edge_map: HashMap<TransitionId, Edge<C, S, T>>,

    entry_state_id: StateId,
    exit_state_id: StateId,

    exited: bool,

    curr_state_id: StateId,
    curr_state_finished: bool,

    state_id_counter: StateId,
    transition_id_counter: TransitionId,
}

impl<C, S: State<C>, T: Transition<C, S>> Fsm<C, S, T> {
    pub fn new(entry_state: S, exit_state: S) -> Fsm<C, S, T> {
        let mut fsm = Fsm {
            _ctx_type: PhantomData,
            node_map: Default::default(),
            edge_map: Default::default(),
            entry_state_id: StateId(0),
            exit_state_id: StateId(0),
            exited: false,
            curr_state_id: StateId(0),
            curr_state_finished: false,
            state_id_counter: StateId(0),
            transition_id_counter: TransitionId(0),
        };
        fsm.entry_state_id = fsm.add_state(entry_state);
        fsm.exit_state_id = fsm.add_state(exit_state);
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
    pub fn curr_state(&self) -> &S {
        &self.curr_node().state
    }
    pub fn curr_state_mut(&mut self) -> &mut S {
        &mut self.curr_node_mut().state
    }

    fn curr_node(&self) -> &Node<C, S> {
        self.node_map.get(&self.curr_state_id).unwrap()
    }
    fn curr_node_mut(&mut self) -> &mut Node<C, S> {
        self.node_map.get_mut(&self.curr_state_id).unwrap()
    }

    pub fn add_state(&mut self, state: S) -> StateId {
        self.state_id_counter.0 += 1;
        self.node_map
            .insert(self.state_id_counter, Node::new(state));
        self.state_id_counter
    }

    pub fn remove_state(&mut self, id: StateId) {
        if let Some(Node {
            _ctx_type: PhantomData,
            in_set,
            out_set,
            state: _,
        }) = self.node_map.remove(&id)
        {
            for id in &in_set {
                self.edge_map.remove(id);
            }
            for id in &out_set {
                self.edge_map.remove(id);
            }
        }
    }

    pub fn add_transition(
        &mut self,
        src_id: StateId,
        dst_id: StateId,
        transition: T,
    ) -> Option<TransitionId> {
        if self.node_map.contains_key(&src_id) && self.node_map.contains_key(&dst_id) {
            self.transition_id_counter.0 += 1;

            let src_node = self.node_map.get_mut(&src_id).unwrap();
            src_node.add_out(self.transition_id_counter);

            let dst_node = self.node_map.get_mut(&dst_id).unwrap();
            dst_node.add_in(self.transition_id_counter);

            self.edge_map.insert(
                self.transition_id_counter,
                Edge::new(src_id, dst_id, transition),
            );
            return Some(self.transition_id_counter);
        }
        None
    }

    pub fn remove_transition(&mut self, id: TransitionId) {
        if let Some(edge) = self.edge_map.remove(&id) {
            let src_node = self.node_map.get_mut(&edge.src_id).unwrap();
            src_node.remove_out(id);

            let dst_state = self.node_map.get_mut(&edge.dst_id).unwrap();
            dst_state.remove_in(id);
        }
    }

    pub fn tick(&mut self, ctx: &mut C) {
        if self.exited {
            return;
        }
        if !self.curr_state_finished {
            self.curr_state_finished = self.curr_node_mut().state.tick(ctx);
        }
        if self.curr_state_finished {
            let curr_node = self.curr_node();
            for transition_id in &curr_node.out_set {
                let edge = &self.edge_map[transition_id];
                let dst_state = &self.node_map[&edge.dst_id].state;
                if edge.transition.satisfied(ctx, &curr_node.state, dst_state) {
                    let next_state_id = edge.dst_id;
                    let curr_state = self.curr_state_mut();
                    curr_state.exit(ctx);

                    self.curr_state_id = next_state_id;
                    self.curr_state_finished = false;
                    let curr_state = self.curr_state_mut();
                    curr_state.enter(ctx);

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

    struct Context;

    enum ActionState {
        Entry,
        Exit,
        Eat {
            food: String,
            food_sum: u32,
            food_count: u32,
        },
        Nap {
            tick_time: u32,
        },
    }

    impl State<Context> for ActionState {
        fn enter(&mut self, _ctx: &mut Context) {
            if let ActionState::Eat {
                food: _,
                food_sum,
                food_count,
            } = self
            {
                // Eat one immediately while entering
                if *food_sum > 0 {
                    *food_count = *food_sum - 1;
                } else {
                    *food_count = 0;
                }
            }
        }

        fn tick(&mut self, _ctx: &mut Context) -> bool {
            match self {
                ActionState::Eat {
                    food: _,
                    food_sum: _,
                    food_count,
                } => {
                    // Eat one per tick
                    if *food_count > 0 {
                        *food_count -= 1;
                    } else {
                        *food_count = 0;
                    }

                    // Eat should not be exited until finishing eating
                    *food_count == 0
                }

                ActionState::Nap { tick_time } => {
                    if *tick_time > 0 {
                        *tick_time = *tick_time - 1;
                    } else {
                        *tick_time = 0;
                    }
                    true
                }
                _ => true,
            }
        }

        fn exit(&mut self, _ctx: &mut Context) {}
    }

    enum ActionTransition {
        Direct,
        EatFinished { food: String },
        NapOnce,
        NapFinished,
    }

    impl Transition<Context, ActionState> for ActionTransition {
        fn satisfied(&self, _ctx: &mut Context, src: &ActionState, _dst: &ActionState) -> bool {
            match self {
                ActionTransition::Direct => true,

                ActionTransition::EatFinished { food: state_food } => {
                    if let ActionState::Eat {
                        food: transition_food,
                        food_sum: _,
                        food_count: _,
                    } = src
                    {
                        return *state_food == *transition_food;
                    }
                    false
                }

                ActionTransition::NapOnce => {
                    if let ActionState::Nap { tick_time } = src {
                        return *tick_time != 0;
                    }
                    false
                }

                ActionTransition::NapFinished => {
                    if let ActionState::Nap { tick_time } = src {
                        return *tick_time == 0;
                    }
                    false
                }
            }
        }
    }

    #[test]
    fn exit_directly() {
        let mut ctx = Context;
        let mut fsm = Fsm::new(ActionState::Entry, ActionState::Exit);
        let entry_id = fsm.entry_state_id();
        let exit_id = fsm.exit_state_id();
        fsm.add_transition(entry_id, exit_id, ActionTransition::Direct);
        assert_eq!(fsm.curr_state_id(), entry_id);
        fsm.tick(&mut ctx);
        assert_eq!(fsm.curr_state_id(), exit_id);
        fsm.tick(&mut ctx);
        assert_eq!(fsm.curr_state_id(), exit_id);
    }

    #[test]
    fn eat_fish_and_then_chip() {
        let mut ctx = Context;
        let mut fsm = Fsm::new(ActionState::Entry, ActionState::Exit);
        let entry_id = fsm.entry_state_id();
        let exit_id = fsm.exit_state_id();
        let eat_fish_id = fsm.add_state(ActionState::Eat {
            food: "Fish".to_string(),
            food_sum: 2,
            food_count: 2,
        });
        let eat_chip_id = fsm.add_state(ActionState::Eat {
            food: "Chip".to_string(),
            food_sum: 3,
            food_count: 3,
        });

        fsm.add_transition(entry_id, eat_fish_id, ActionTransition::Direct);
        fsm.add_transition(
            eat_fish_id,
            eat_chip_id,
            ActionTransition::EatFinished {
                food: "Fish".to_string(),
            },
        );
        fsm.add_transition(
            eat_chip_id,
            exit_id,
            ActionTransition::EatFinished {
                food: "Chip".to_string(),
            },
        );

        assert_eq!(fsm.curr_state_id(), entry_id);
        fsm.tick(&mut ctx);
        assert_eq!(fsm.curr_state_id(), eat_fish_id);
        fsm.tick(&mut ctx);
        assert_eq!(fsm.curr_state_id(), eat_chip_id);
        fsm.tick(&mut ctx);
        assert_eq!(fsm.curr_state_id(), eat_chip_id);
        fsm.tick(&mut ctx);
        assert_eq!(fsm.curr_state_id(), exit_id);
    }

    #[test]
    fn eat_fish_and_then_nap() {
        let mut ctx = Context;
        let mut fsm = Fsm::new(ActionState::Entry, ActionState::Exit);
        let entry_id = fsm.entry_state_id();
        let exit_id = fsm.exit_state_id();
        let eat_fish_id = fsm.add_state(ActionState::Eat {
            food: "Fish".to_string(),
            food_sum: 2,
            food_count: 2,
        });
        let nap_id = fsm.add_state(ActionState::Nap { tick_time: 2 });

        fsm.add_transition(entry_id, eat_fish_id, ActionTransition::Direct);
        fsm.add_transition(
            eat_fish_id,
            nap_id,
            ActionTransition::EatFinished {
                food: "Fish".to_string(),
            },
        );
        fsm.add_transition(nap_id, exit_id, ActionTransition::NapOnce);

        assert_eq!(fsm.curr_state_id(), entry_id);
        fsm.tick(&mut ctx);
        assert_eq!(fsm.curr_state_id(), eat_fish_id);
        fsm.tick(&mut ctx);
        assert_eq!(fsm.curr_state_id(), nap_id);
        fsm.tick(&mut ctx);
        assert_eq!(fsm.curr_state_id(), exit_id);
    }

    #[test]
    fn eat_and_nap_by_turn() {
        let mut ctx = Context;
        let mut fsm = Fsm::new(ActionState::Entry, ActionState::Exit);
        let entry_id = fsm.entry_state_id();
        let exit_id = fsm.exit_state_id();
        let eat_fish_id = fsm.add_state(ActionState::Eat {
            food: "Fish".to_string(),
            food_sum: 3,
            food_count: 3,
        });
        let nap_id = fsm.add_state(ActionState::Nap { tick_time: 2 });

        fsm.add_transition(entry_id, eat_fish_id, ActionTransition::Direct);
        fsm.add_transition(
            eat_fish_id,
            nap_id,
            ActionTransition::EatFinished {
                food: "Fish".to_string(),
            },
        );
        fsm.add_transition(nap_id, eat_fish_id, ActionTransition::NapOnce);
        fsm.add_transition(nap_id, exit_id, ActionTransition::NapFinished);

        assert_eq!(fsm.curr_state_id(), entry_id);
        fsm.tick(&mut ctx);
        assert_eq!(fsm.curr_state_id(), eat_fish_id);
        fsm.tick(&mut ctx);
        assert_eq!(fsm.curr_state_id(), eat_fish_id);
        fsm.tick(&mut ctx);
        assert_eq!(fsm.curr_state_id(), nap_id);
        fsm.tick(&mut ctx);
        assert_eq!(fsm.curr_state_id(), eat_fish_id);
        fsm.tick(&mut ctx);
        assert_eq!(fsm.curr_state_id(), eat_fish_id);
        fsm.tick(&mut ctx);
        assert_eq!(fsm.curr_state_id(), nap_id);
        fsm.tick(&mut ctx);
        assert_eq!(fsm.curr_state_id(), exit_id);
    }
}
