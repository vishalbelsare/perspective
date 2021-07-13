////////////////////////////////////////////////////////////////////////////////
//
// Copyright (c) 2018, the Perspective Authors.
//
// This file is part of the Perspective library, distributed under the terms
// of the Apache License 2.0.  The full license can be found in the LICENSE
// file.

use crate::config::*;
use crate::utils::*;

use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::*;
use yew::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DropAction {
    Active,
    RowPivots,
    ColumnPivots,
    Sort,
    Filter,
}

#[derive(Clone, Debug)]
pub struct DragState {
    column: String,
    state: Option<(DropAction, usize)>,
}

#[derive(Clone)]
pub struct DragDropState {
    drag_state: Option<DragState>,
    on_drop_action: PubSub<(String, DropAction, usize)>,
}

/// The `<perspective-viewer>` drag-drop service, which manages drag/drop user
/// interactions across components.  It is a component-level service, since only one
/// drag/drop action can be executed by the user at a time, and has a 3 states:
/// - `None` No drag/drop action is in effect.
/// - `Some(DragDropState { state: None }` Drag is in effect.
/// - `Some(DragDropState { state: Some(_) }` Drag and Hover are in effect.
#[derive(Clone)]
pub struct DragDrop(Rc<RefCell<DragDropState>>);

impl Deref for DragDrop {
    type Target = Rc<RefCell<DragDropState>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for DragDrop {
    fn default() -> Self {
        DragDrop(Rc::new(RefCell::new(DragDropState {
            drag_state: None,
            on_drop_action: PubSub::default(),
        })))
    }
}

impl DragDrop {
    pub fn add_on_drop_action(
        &self,
        callback: Callback<(String, DropAction, usize)>,
    ) -> Subscription {
        self.borrow().on_drop_action.add_listener(callback)
    }

    pub fn notify_drop(&self) {
        let action = match self.borrow().drag_state {
            Some(DragState {
                ref column,
                state: Some((action, index)),
                ..
            }) => Some((column.to_owned(), action, index)),
            _ => None,
        };

        if let Some(action) = action {
            let pubsub = self.borrow().on_drop_action.clone();
            pubsub.emit_all(action);
        }

        self.drag_end();
    }

    /// Get the column name currently being drag/dropped.
    pub fn get_drag_column(&self) -> Option<String> {
        match self.borrow().drag_state {
            Some(DragState { ref column, .. }) => Some(column.clone()),
            _ => None,
        }
    }

    /// Start the drag/drop action with the name of the column being dragged.
    pub fn drag_start(&self, column: String) {
        self.borrow_mut().drag_state = Some(DragState {
            column,
            state: None,
        });
    }

    /// End the drag/drop action by resetting the state to default.
    pub fn drag_end(&self) {
        self.borrow_mut().drag_state = None;
    }

    /// Leave the `action` zone.
    pub fn drag_leave(&self, action: DropAction) {
        let reset = match self.borrow().drag_state {
            Some(DragState {
                ref column,
                state: Some((a, _)),
            }) if a == action => Some(column.clone()),
            _ => None,
        };

        if let Some(column) = reset {
            self.drag_start(column);
        }
    }

    // Enter the `action` zone at `index`, which must be <= the number of children
    // in the container.
    pub fn drag_enter(&self, action: DropAction, index: usize) -> bool {
        let mut r = self.borrow_mut();
        let should_render = match r.drag_state {
            Some(DragState {
                state: Some((a, x)),
                ..
            }) if a == action => x != index,
            _ => true,
        };

        r.drag_state
            .as_mut()
            .expect("Hover index without hover")
            .state = Some((action, index));

        should_render
    }

    // Is the drag/drop state currently in `action`?
    pub fn is_dragover(&self, action: DropAction) -> Option<(usize, String)> {
        match self.borrow().drag_state {
            Some(DragState {
                ref column,
                state: Some((a, index)),
                ..
            }) if a == action => Some((index, column.clone())),
            _ => None,
        }
    }
}

/// HTML drag/drop will fire a bubbling `dragleave` event over all children of a
/// `dragleave`-listened-to element, so we need to filter out the events from the
/// children elements with this esoteric DOM arcana.
pub fn dragleave_helper(callback: impl Fn() + 'static) -> Callback<DragEvent> {
    Callback::from({
        move |event: DragEvent| {
            let related_target = event
                .related_target()
                .or_else(|| Some(JsValue::UNDEFINED.unchecked_into::<EventTarget>()))
                .map(|x| x.unchecked_into::<Node>());

            let current_target = event
                .current_target()
                .unwrap()
                .unchecked_into::<HtmlElement>();

            if !current_target.contains(related_target.as_ref()) {
                callback();
            }
        }
    })
}
