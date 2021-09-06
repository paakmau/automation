use std::thread::sleep;
use std::time::Duration;

use automation::context::{Context, MouseButton};
use automation::fsm::{Fsm, PresetState, PresetTransition};
use automation::image::Pattern;

fn main() {
    let from_file_buf = Pattern::from_file_buf;

    let ok_blue = from_file_buf(include_bytes!("patterns/ok_blue.png")).unwrap();
    let ok_white = from_file_buf(include_bytes!("patterns/ok_white.png")).unwrap();
    let back = from_file_buf(include_bytes!("patterns/back.png")).unwrap();
    let start = from_file_buf(include_bytes!("patterns/start.png")).unwrap();
    let team_empty = from_file_buf(include_bytes!("patterns/team_empty.png")).unwrap();
    let team = from_file_buf(include_bytes!("patterns/team.png")).unwrap();
    let use_team = from_file_buf(include_bytes!("patterns/use_team.png")).unwrap();
    let begin_battle = from_file_buf(include_bytes!("patterns/begin_battle.png")).unwrap();
    let next = from_file_buf(include_bytes!("patterns/next.png")).unwrap();

    let add = from_file_buf(include_bytes!("patterns/add.png")).unwrap();
    let use_one = from_file_buf(include_bytes!("patterns/use_one.png")).unwrap();
    let use_two = from_file_buf(include_bytes!("patterns/use_two.png")).unwrap();

    let quests = from_file_buf(include_bytes!("patterns/quests.png")).unwrap();
    let quests_curr = from_file_buf(include_bytes!("patterns/quests_curr.png")).unwrap();

    let expl = from_file_buf(include_bytes!("patterns/expl.png")).unwrap();
    let expl_left_zero = from_file_buf(include_bytes!("patterns/expl_left_zero.png")).unwrap();
    let expl_left_two = from_file_buf(include_bytes!("patterns/expl_left_two.png")).unwrap();
    let expl_lv = from_file_buf(include_bytes!("patterns/expl_lv.png")).unwrap();
    let back_to_expl = from_file_buf(include_bytes!("patterns/back_to_expl.png")).unwrap();

    let expl_exp = from_file_buf(include_bytes!("patterns/expl_exp.png")).unwrap();
    let expl_mana = from_file_buf(include_bytes!("patterns/expl_mana.png")).unwrap();

    let dungeon = from_file_buf(include_bytes!("patterns/dungeon.png")).unwrap();
    let dungeon_ex = from_file_buf(include_bytes!("patterns/dungeon_ex.png")).unwrap();
    let dungeon_left_zero =
        from_file_buf(include_bytes!("patterns/dungeon_left_zero.png")).unwrap();
    let dungeon_left_one = from_file_buf(include_bytes!("patterns/dungeon_left_one.png")).unwrap();
    let dungeon_box = from_file_buf(include_bytes!("patterns/dungeon_box.png")).unwrap();
    let dungeon_fail = from_file_buf(include_bytes!("patterns/dungeon_fail.png")).unwrap();

    let mut ctx = Context::new();
    let mut fsm = Fsm::new(PresetState::Entry, PresetState::Exit);

    let entry = fsm.entry_state_id();
    let exit = fsm.exit_state_id();

    use automation::fsm::PresetState::Emtpy;
    use automation::image::Direction::{Down, Up};
    fn clk_at(pattern: &Pattern) -> PresetState {
        PresetState::MouseClickAt {
            pattern,
            dir: Up,
            btn: MouseButton::Left,
        }
    }
    fn clk_at_down(pattern: &Pattern) -> PresetState {
        PresetState::MouseClickAt {
            pattern,
            dir: Down,
            btn: MouseButton::Left,
        }
    }

    use automation::fsm::PresetTransition::Direct;
    fn found(pattern: &Pattern) -> PresetTransition {
        PresetTransition::PatternFound { pattern, dir: Up }
    }

    // The exploration panel
    let clk_quests = fsm.add_state(clk_at(&quests));
    let clk_quests_curr = fsm.add_state(clk_at(&quests_curr));
    let clk_expl = fsm.add_state(clk_at(&expl));
    fsm.add_transition(entry, clk_quests, found(&quests));
    fsm.add_transition(entry, clk_quests_curr, found(&quests_curr));
    fsm.add_transition(clk_quests, clk_expl, Direct);
    fsm.add_transition(clk_quests_curr, clk_expl, Direct);

    // The exp level
    let clk_expl_exp = fsm.add_state(clk_at(&expl_exp));
    fsm.add_transition(clk_expl, clk_expl_exp, Direct);

    let back_at_expl_after_exp = fsm.add_state(Emtpy);

    // If there's no attempt left, back to expl panel
    let back_to_expl_after_exp = fsm.add_state(clk_at(&back));
    fsm.add_transition(clk_expl_exp, back_to_expl_after_exp, found(&expl_left_zero));
    fsm.add_transition(back_to_expl_after_exp, back_at_expl_after_exp, Direct);
    // If there's 2 attempt left, enter and then use 2 tickets
    let clk_expl_exp_lv = fsm.add_state(clk_at(&expl_lv));
    let at_expl_exp_lv = fsm.add_state(Emtpy);
    let clk_expl_exp_lv_add = fsm.add_state(clk_at(&add));
    let clk_expl_exp_lv_use_two = fsm.add_state(clk_at(&use_two));
    let clk_expl_exp_lv_use_two_ok = fsm.add_state(clk_at(&ok_blue));
    let clk_back_to_expl_after_exp = fsm.add_state(clk_at(&back_to_expl));
    fsm.add_transition(clk_expl_exp, clk_expl_exp_lv, found(&expl_left_two));
    fsm.add_transition(clk_expl_exp_lv, at_expl_exp_lv, Direct);
    fsm.add_transition(at_expl_exp_lv, clk_expl_exp_lv_add, found(&use_one));
    fsm.add_transition(clk_expl_exp_lv_add, at_expl_exp_lv, Direct);
    fsm.add_transition(at_expl_exp_lv, clk_expl_exp_lv_use_two, found(&use_two));
    fsm.add_transition(clk_expl_exp_lv_use_two, clk_expl_exp_lv_use_two_ok, Direct);
    fsm.add_transition(
        clk_expl_exp_lv_use_two_ok,
        clk_back_to_expl_after_exp,
        Direct,
    );
    fsm.add_transition(clk_back_to_expl_after_exp, back_at_expl_after_exp, Direct);

    // Now consider the mana level
    let clk_expl_mana = fsm.add_state(clk_at(&expl_mana));
    fsm.add_transition(back_at_expl_after_exp, clk_expl_mana, Direct);

    let back_at_expl_after_mana = fsm.add_state(Emtpy);

    // If there's no attempt left, back to expl panel
    let back_to_expl_after_mana = fsm.add_state(clk_at(&back));
    fsm.add_transition(
        clk_expl_mana,
        back_to_expl_after_mana,
        found(&expl_left_zero),
    );
    fsm.add_transition(back_to_expl_after_mana, back_at_expl_after_mana, Direct);

    // If there's 2 attempt left, enter and then use 2 tickets
    let clk_expl_mana_lv = fsm.add_state(clk_at(&expl_lv));
    let at_expl_mana_lv = fsm.add_state(Emtpy);
    let clk_expl_mana_lv_add = fsm.add_state(clk_at(&add));
    let clk_expl_mana_lv_use_two = fsm.add_state(clk_at(&use_two));
    let clk_expl_mana_lv_use_two_ok = fsm.add_state(clk_at(&ok_blue));
    let clk_back_to_expl_after_mana = fsm.add_state(clk_at(&back_to_expl));
    fsm.add_transition(clk_expl_mana, clk_expl_mana_lv, found(&expl_left_two));
    fsm.add_transition(clk_expl_mana_lv, at_expl_mana_lv, Direct);
    fsm.add_transition(at_expl_mana_lv, clk_expl_mana_lv_add, found(&use_one));
    fsm.add_transition(clk_expl_mana_lv_add, at_expl_mana_lv, Direct);
    fsm.add_transition(at_expl_mana_lv, clk_expl_mana_lv_use_two, found(&use_two));
    fsm.add_transition(
        clk_expl_mana_lv_use_two,
        clk_expl_mana_lv_use_two_ok,
        Direct,
    );
    fsm.add_transition(
        clk_expl_mana_lv_use_two_ok,
        clk_back_to_expl_after_mana,
        Direct,
    );
    fsm.add_transition(clk_back_to_expl_after_mana, back_at_expl_after_mana, Direct);

    // Back to quests after expl
    let back_to_quests_after_expl = fsm.add_state(clk_at(&back));
    fsm.add_transition(back_at_expl_after_mana, back_to_quests_after_expl, Direct);

    // The dungeon
    let clk_dungeon = fsm.add_state(clk_at(&dungeon));
    let clk_dungeon_ex = fsm.add_state(clk_at(&dungeon_ex));
    let clk_dungeon_ex_ok = fsm.add_state(clk_at(&ok_blue));
    let at_dungeon_ex = fsm.add_state(Emtpy);
    fsm.add_transition(back_to_quests_after_expl, clk_dungeon, Direct);
    fsm.add_transition(clk_dungeon, at_dungeon_ex, found(&dungeon_box));
    fsm.add_transition(clk_dungeon, clk_dungeon_ex, found(&dungeon_left_one));
    fsm.add_transition(clk_dungeon, exit, found(&dungeon_left_zero));
    fsm.add_transition(clk_dungeon_ex, clk_dungeon_ex_ok, Direct);
    fsm.add_transition(clk_dungeon_ex_ok, at_dungeon_ex, Direct);

    // If box found, click it
    let clk_dungeon_box = fsm.add_state(clk_at_down(&dungeon_box));
    let at_dungeon_box = fsm.add_state(Emtpy);
    let clk_dungeon_box_start = fsm.add_state(clk_at(&start));
    fsm.add_transition(at_dungeon_ex, clk_dungeon_box, Direct);
    fsm.add_transition(clk_dungeon_box, at_dungeon_box, Direct);
    fsm.add_transition(at_dungeon_box, clk_dungeon_box, found(&dungeon_box));
    fsm.add_transition(at_dungeon_box, clk_dungeon_box_start, found(&start));

    // If team not used, choose one
    let clk_dungeon_box_team = fsm.add_state(clk_at(&team));
    let clk_dungeon_box_use_team = fsm.add_state(clk_at(&use_team));
    let clk_dungeon_box_begin_battle = fsm.add_state(clk_at(&begin_battle));
    fsm.add_transition(
        clk_dungeon_box_start,
        clk_dungeon_box_begin_battle,
        found(&begin_battle),
    );
    fsm.add_transition(
        clk_dungeon_box_start,
        clk_dungeon_box_team,
        found(&team_empty),
    );
    fsm.add_transition(clk_dungeon_box_team, clk_dungeon_box_use_team, Direct);
    fsm.add_transition(
        clk_dungeon_box_use_team,
        clk_dungeon_box_begin_battle,
        Direct,
    );

    // Back to dungeon ex after a battle
    let clk_dungeon_box_fail = fsm.add_state(clk_at(&dungeon_fail));
    let clk_dungeon_box_next = fsm.add_state(clk_at(&next));
    let clk_dungeon_box_ok = fsm.add_state(clk_at(&ok_white));
    let at_dungeon_box_ok = fsm.add_state(Emtpy);
    fsm.add_transition(
        clk_dungeon_box_begin_battle,
        clk_dungeon_box_fail,
        found(&dungeon_fail),
    );
    fsm.add_transition(clk_dungeon_box_fail, at_dungeon_ex, Direct);
    fsm.add_transition(
        clk_dungeon_box_begin_battle,
        clk_dungeon_box_next,
        found(&next),
    );
    fsm.add_transition(clk_dungeon_box_next, clk_dungeon_box_ok, Direct);
    fsm.add_transition(clk_dungeon_box_ok, at_dungeon_box_ok, Direct);
    fsm.add_transition(at_dungeon_box_ok, at_dungeon_ex, found(&dungeon_box));
    fsm.add_transition(at_dungeon_box_ok, exit, found(&dungeon_ex));

    while fsm.curr_state_id() != exit {
        fsm.tick(&mut ctx);
        sleep(Duration::from_millis(200));
    }
}
