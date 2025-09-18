use std::collections::HashMap;
use std::sync::{Arc, RwLock, Weak};
use std::any::{Any, TypeId};
use crate::engine::error::RobinResult;
use serde::{Serialize, Deserialize};

/// Global state management for UI components
pub struct StateManager {
    stores: HashMap<TypeId, Arc<RwLock<dyn StateStore>>>,
    contexts: HashMap<String, Arc<RwLock<Context>>>,
    subscriptions: HashMap<TypeId, Vec<Subscription>>,
}

/// Trait for state stores
pub trait StateStore: Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_store(&self) -> Box<dyn StateStore>;
}

/// Context for passing data down the component tree
#[derive(Debug, Clone)]
pub struct Context {
    values: HashMap<String, ContextValue>,
    parent: Option<Arc<RwLock<Context>>>,
}

#[derive(Debug, Clone)]
pub enum ContextValue {
    String(String),
    Number(f64),
    Bool(bool),
    Data(Arc<RwLock<dyn Any + Send + Sync>>),
}

/// Subscription to state changes
pub struct Subscription {
    id: u64,
    callback: Arc<dyn Fn() + Send + Sync>,
    weak_ref: Weak<RwLock<dyn StateStore>>,
}

/// Hook-like state management for components
pub struct UseState<T: Clone> {
    value: Arc<RwLock<T>>,
    listeners: Arc<RwLock<Vec<StateListener<T>>>>,
}

pub struct StateListener<T> {
    id: u64,
    callback: Arc<dyn Fn(&T) + Send + Sync>,
}

impl<T: Clone + 'static> UseState<T> {
    pub fn new(initial: T) -> Self {
        Self {
            value: Arc::new(RwLock::new(initial)),
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get(&self) -> T {
        self.value.read().unwrap().clone()
    }

    pub fn set(&self, new_value: T) {
        *self.value.write().unwrap() = new_value.clone();
        self.notify_listeners(&new_value);
    }

    pub fn update<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        let mut value = self.value.write().unwrap();
        updater(&mut *value);
        let new_value = value.clone();
        drop(value);
        self.notify_listeners(&new_value);
    }

    pub fn subscribe<F>(&self, callback: F) -> u64
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        let id = self.next_listener_id();
        let listener = StateListener {
            id,
            callback: Arc::new(callback),
        };
        self.listeners.write().unwrap().push(listener);
        id
    }

    pub fn unsubscribe(&self, id: u64) {
        self.listeners.write().unwrap().retain(|l| l.id != id);
    }

    fn notify_listeners(&self, value: &T) {
        let listeners = self.listeners.read().unwrap();
        for listener in listeners.iter() {
            (listener.callback)(value);
        }
    }

    fn next_listener_id(&self) -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        COUNTER.fetch_add(1, Ordering::SeqCst)
    }
}

/// Reducer-based state management (like Redux)
pub struct UseReducer<S: Clone, A> {
    state: Arc<RwLock<S>>,
    reducer: Arc<dyn Fn(&S, A) -> S + Send + Sync>,
    listeners: Arc<RwLock<Vec<Arc<dyn Fn(&S) + Send + Sync>>>>,
}

impl<S: Clone + 'static, A: 'static> UseReducer<S, A> {
    pub fn new<F>(initial: S, reducer: F) -> Self
    where
        F: Fn(&S, A) -> S + Send + Sync + 'static,
    {
        Self {
            state: Arc::new(RwLock::new(initial)),
            reducer: Arc::new(reducer),
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get_state(&self) -> S {
        self.state.read().unwrap().clone()
    }

    pub fn dispatch(&self, action: A) {
        let current_state = self.state.read().unwrap().clone();
        let new_state = (self.reducer)(&current_state, action);
        *self.state.write().unwrap() = new_state.clone();
        self.notify_listeners(&new_state);
    }

    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&S) + Send + Sync + 'static,
    {
        self.listeners.write().unwrap().push(Arc::new(callback));
    }

    fn notify_listeners(&self, state: &S) {
        let listeners = self.listeners.read().unwrap();
        for listener in listeners.iter() {
            listener(state);
        }
    }
}

/// Effect hook for side effects
pub struct UseEffect {
    cleanup: Option<Arc<dyn Fn() + Send + Sync>>,
    dependencies: Vec<DependencyValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DependencyValue {
    String(String),
    Number(f64),
    Bool(bool),
}

impl UseEffect {
    pub fn new<F, C>(effect: F, cleanup: Option<C>, deps: Vec<DependencyValue>) -> Self
    where
        F: FnOnce() + 'static,
        C: Fn() + Send + Sync + 'static,
    {
        effect();

        Self {
            cleanup: cleanup.map(|c| Arc::new(c) as Arc<dyn Fn() + Send + Sync>),
            dependencies: deps,
        }
    }

    pub fn update<F, C>(&mut self, effect: F, cleanup: Option<C>, deps: Vec<DependencyValue>)
    where
        F: FnOnce() + 'static,
        C: Fn() + Send + Sync + 'static,
    {
        if self.dependencies != deps {
            // Run cleanup from previous effect
            if let Some(ref cleanup_fn) = self.cleanup {
                cleanup_fn();
            }

            // Run new effect
            effect();

            // Update state
            self.cleanup = cleanup.map(|c| Arc::new(c) as Arc<dyn Fn() + Send + Sync>);
            self.dependencies = deps;
        }
    }
}

impl Drop for UseEffect {
    fn drop(&mut self) {
        if let Some(ref cleanup) = self.cleanup {
            cleanup();
        }
    }
}

/// Memoization hook for expensive computations
pub struct UseMemo<T: Clone> {
    value: Arc<RwLock<T>>,
    dependencies: Vec<DependencyValue>,
}

impl<T: Clone + 'static> UseMemo<T> {
    pub fn new<F>(compute: F, deps: Vec<DependencyValue>) -> Self
    where
        F: FnOnce() -> T,
    {
        Self {
            value: Arc::new(RwLock::new(compute())),
            dependencies: deps,
        }
    }

    pub fn get(&self) -> T {
        self.value.read().unwrap().clone()
    }

    pub fn update<F>(&mut self, compute: F, deps: Vec<DependencyValue>)
    where
        F: FnOnce() -> T,
    {
        if self.dependencies != deps {
            *self.value.write().unwrap() = compute();
            self.dependencies = deps;
        }
    }
}

/// Callback memoization hook
pub struct UseCallback<F> {
    callback: Arc<F>,
    dependencies: Vec<DependencyValue>,
}

impl<F: 'static> UseCallback<F> {
    pub fn new(callback: F, deps: Vec<DependencyValue>) -> Self {
        Self {
            callback: Arc::new(callback),
            dependencies: deps,
        }
    }

    pub fn get(&self) -> Arc<F> {
        Arc::clone(&self.callback)
    }

    pub fn update(&mut self, callback: F, deps: Vec<DependencyValue>) {
        if self.dependencies != deps {
            self.callback = Arc::new(callback);
            self.dependencies = deps;
        }
    }
}

/// Reference hook for accessing DOM elements or storing mutable values
pub struct UseRef<T> {
    current: Arc<RwLock<Option<T>>>,
}

impl<T: 'static> UseRef<T> {
    pub fn new() -> Self {
        Self {
            current: Arc::new(RwLock::new(None)),
        }
    }

    pub fn get(&self) -> Option<T>
    where
        T: Clone,
    {
        self.current.read().unwrap().clone()
    }

    pub fn set(&self, value: T) {
        *self.current.write().unwrap() = Some(value);
    }

    pub fn clear(&self) {
        *self.current.write().unwrap() = None;
    }
}

/// Global state store implementation
#[derive(Clone)]
pub struct Store<T: Clone + Send + Sync + 'static> {
    state: Arc<RwLock<T>>,
    subscribers: Arc<RwLock<Vec<Arc<dyn Fn(&T) + Send + Sync>>>>,
}

impl<T: Clone + Send + Sync + 'static> Store<T> {
    pub fn new(initial: T) -> Self {
        Self {
            state: Arc::new(RwLock::new(initial)),
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get_state(&self) -> T {
        self.state.read().unwrap().clone()
    }

    pub fn set_state(&self, new_state: T) {
        *self.state.write().unwrap() = new_state.clone();
        self.notify_subscribers(&new_state);
    }

    pub fn update_state<F>(&self, updater: F)
    where
        F: FnOnce(&mut T),
    {
        let mut state = self.state.write().unwrap();
        updater(&mut *state);
        let new_state = state.clone();
        drop(state);
        self.notify_subscribers(&new_state);
    }

    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&T) + Send + Sync + 'static,
    {
        self.subscribers.write().unwrap().push(Arc::new(callback));
    }

    fn notify_subscribers(&self, state: &T) {
        let subscribers = self.subscribers.read().unwrap();
        for subscriber in subscribers.iter() {
            subscriber(state);
        }
    }
}

/// Form state management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormState {
    pub values: HashMap<String, FormValue>,
    pub errors: HashMap<String, String>,
    pub touched: HashMap<String, bool>,
    pub is_submitting: bool,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormValue {
    String(String),
    Number(f64),
    Bool(bool),
    Array(Vec<FormValue>),
}

pub struct UseForm {
    state: Arc<RwLock<FormState>>,
    validators: HashMap<String, Arc<dyn Fn(&FormValue) -> Option<String> + Send + Sync>>,
    listeners: Arc<RwLock<Vec<Arc<dyn Fn(&FormState) + Send + Sync>>>>,
}

impl UseForm {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(FormState {
                values: HashMap::new(),
                errors: HashMap::new(),
                touched: HashMap::new(),
                is_submitting: false,
                is_valid: true,
            })),
            validators: HashMap::new(),
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn get_value(&self, field: &str) -> Option<FormValue> {
        self.state.read().unwrap().values.get(field).cloned()
    }

    pub fn set_value(&self, field: &str, value: FormValue) {
        let mut state = self.state.write().unwrap();
        state.values.insert(field.to_string(), value.clone());

        // Run validation
        if let Some(validator) = self.validators.get(field) {
            if let Some(error) = validator(&value) {
                state.errors.insert(field.to_string(), error);
                state.is_valid = false;
            } else {
                state.errors.remove(field);
                state.is_valid = state.errors.is_empty();
            }
        }

        let new_state = state.clone();
        drop(state);
        self.notify_listeners(&new_state);
    }

    pub fn set_field_touched(&self, field: &str, touched: bool) {
        let mut state = self.state.write().unwrap();
        state.touched.insert(field.to_string(), touched);
        let new_state = state.clone();
        drop(state);
        self.notify_listeners(&new_state);
    }

    pub fn add_validator<F>(&mut self, field: &str, validator: F)
    where
        F: Fn(&FormValue) -> Option<String> + Send + Sync + 'static,
    {
        self.validators.insert(field.to_string(), Arc::new(validator));
    }

    pub fn validate_all(&self) -> bool {
        let mut state = self.state.write().unwrap();
        state.errors.clear();
        state.is_valid = true;

        // Collect field-value pairs to avoid borrow conflict
        let field_values: Vec<_> = state.values.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

        for (field, value) in field_values {
            if let Some(validator) = self.validators.get(&field) {
                if let Some(error) = validator(&value) {
                    state.errors.insert(field, error);
                    state.is_valid = false;
                }
            }
        }

        state.is_valid
    }

    pub fn reset(&self) {
        let mut state = self.state.write().unwrap();
        state.values.clear();
        state.errors.clear();
        state.touched.clear();
        state.is_submitting = false;
        state.is_valid = true;
        let new_state = state.clone();
        drop(state);
        self.notify_listeners(&new_state);
    }

    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&FormState) + Send + Sync + 'static,
    {
        self.listeners.write().unwrap().push(Arc::new(callback));
    }

    fn notify_listeners(&self, state: &FormState) {
        let listeners = self.listeners.read().unwrap();
        for listener in listeners.iter() {
            listener(state);
        }
    }
}

/// Router state for navigation
#[derive(Debug, Clone)]
pub struct RouterState {
    pub current_path: String,
    pub params: HashMap<String, String>,
    pub query: HashMap<String, String>,
    pub history: Vec<String>,
}

pub struct UseRouter {
    state: Arc<RwLock<RouterState>>,
    listeners: Arc<RwLock<Vec<Arc<dyn Fn(&RouterState) + Send + Sync>>>>,
}

impl UseRouter {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(RouterState {
                current_path: "/".to_string(),
                params: HashMap::new(),
                query: HashMap::new(),
                history: vec!["/".to_string()],
            })),
            listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn navigate(&self, path: &str) {
        let mut state = self.state.write().unwrap();
        let current_path = state.current_path.clone();
        state.history.push(current_path);
        state.current_path = path.to_string();

        // Parse query parameters
        if let Some(query_start) = path.find('?') {
            let query_string = &path[query_start + 1..];
            state.query = self.parse_query_string(query_string);
        } else {
            state.query.clear();
        }

        let new_state = state.clone();
        drop(state);
        self.notify_listeners(&new_state);
    }

    pub fn go_back(&self) {
        let mut state = self.state.write().unwrap();
        if let Some(previous) = state.history.pop() {
            state.current_path = previous;
            let new_state = state.clone();
            drop(state);
            self.notify_listeners(&new_state);
        }
    }

    pub fn get_current_path(&self) -> String {
        self.state.read().unwrap().current_path.clone()
    }

    pub fn get_param(&self, key: &str) -> Option<String> {
        self.state.read().unwrap().params.get(key).cloned()
    }

    pub fn get_query(&self, key: &str) -> Option<String> {
        self.state.read().unwrap().query.get(key).cloned()
    }

    pub fn subscribe<F>(&self, callback: F)
    where
        F: Fn(&RouterState) + Send + Sync + 'static,
    {
        self.listeners.write().unwrap().push(Arc::new(callback));
    }

    fn parse_query_string(&self, query: &str) -> HashMap<String, String> {
        let mut params = HashMap::new();
        for pair in query.split('&') {
            if let Some(eq_pos) = pair.find('=') {
                let key = &pair[..eq_pos];
                let value = &pair[eq_pos + 1..];
                params.insert(key.to_string(), value.to_string());
            }
        }
        params
    }

    fn notify_listeners(&self, state: &RouterState) {
        let listeners = self.listeners.read().unwrap();
        for listener in listeners.iter() {
            listener(state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_use_state() {
        let state = UseState::new(0);
        assert_eq!(state.get(), 0);

        state.set(5);
        assert_eq!(state.get(), 5);

        state.update(|v| *v += 10);
        assert_eq!(state.get(), 15);
    }

    #[test]
    fn test_use_reducer() {
        #[derive(Clone)]
        enum Action {
            Increment,
            Decrement,
            Reset,
        }

        let reducer = UseReducer::new(0, |state, action| match action {
            Action::Increment => state + 1,
            Action::Decrement => state - 1,
            Action::Reset => 0,
        });

        assert_eq!(reducer.get_state(), 0);

        reducer.dispatch(Action::Increment);
        assert_eq!(reducer.get_state(), 1);

        reducer.dispatch(Action::Decrement);
        assert_eq!(reducer.get_state(), 0);

        reducer.dispatch(Action::Reset);
        assert_eq!(reducer.get_state(), 0);
    }

    #[test]
    fn test_use_memo() {
        let mut memo = UseMemo::new(|| 2 + 2, vec![DependencyValue::Number(1.0)]);
        assert_eq!(memo.get(), 4);

        // Same dependencies, should not recompute
        memo.update(|| 3 + 3, vec![DependencyValue::Number(1.0)]);
        assert_eq!(memo.get(), 4);

        // Different dependencies, should recompute
        memo.update(|| 3 + 3, vec![DependencyValue::Number(2.0)]);
        assert_eq!(memo.get(), 6);
    }

    #[test]
    fn test_form_state() {
        let mut form = UseForm::new();

        // Add email validator
        form.add_validator("email", |value| {
            if let FormValue::String(s) = value {
                if !s.contains('@') {
                    return Some("Invalid email".to_string());
                }
            }
            None
        });

        // Set invalid email
        form.set_value("email", FormValue::String("invalid".to_string()));
        assert!(!form.state.read().unwrap().is_valid);

        // Set valid email
        form.set_value("email", FormValue::String("test@example.com".to_string()));
        assert!(form.state.read().unwrap().is_valid);

        // Reset form
        form.reset();
        assert!(form.state.read().unwrap().values.is_empty());
    }

    #[test]
    fn test_router() {
        let router = UseRouter::new();
        assert_eq!(router.get_current_path(), "/");

        router.navigate("/about?id=123");
        assert_eq!(router.get_current_path(), "/about?id=123");
        assert_eq!(router.get_query("id"), Some("123".to_string()));

        router.navigate("/contact");
        assert_eq!(router.get_current_path(), "/contact");

        router.go_back();
        assert_eq!(router.get_current_path(), "/about?id=123");
    }
}