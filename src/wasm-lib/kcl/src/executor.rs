//! The executor for the AST.

use std::collections::HashMap;

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    abstract_syntax_tree_types::{BodyItem, FunctionExpression, Value},
    engine::EngineConnection,
    errors::{KclError, KclErrorDetails},
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ts_rs::TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ProgramMemory {
    pub root: HashMap<String, MemoryItem>,
    #[serde(rename = "return")]
    pub return_: Option<ProgramReturn>,
}

impl ProgramMemory {
    pub fn new() -> Self {
        Self {
            root: HashMap::new(),
            return_: None,
        }
    }

    /// Add to the program memory.
    pub fn add(&mut self, key: &str, value: MemoryItem, source_range: SourceRange) -> Result<(), KclError> {
        if self.root.get(key).is_some() {
            return Err(KclError::ValueAlreadyDefined(KclErrorDetails {
                message: format!("Cannot redefine {}", key),
                source_ranges: vec![source_range],
            }));
        }

        self.root.insert(key.to_string(), value);

        Ok(())
    }

    /// Get a value from the program memory.
    pub fn get(&self, key: &str, source_range: SourceRange) -> Result<&MemoryItem, KclError> {
        self.root.get(key).ok_or_else(|| {
            KclError::UndefinedValue(KclErrorDetails {
                message: format!("memory item key `{}` is not defined", key),
                source_ranges: vec![source_range],
            })
        })
    }
}

impl Default for ProgramMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ts_rs::TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ProgramReturn {
    Arguments(Vec<Value>),
    Value(MemoryItem),
}

impl From<ProgramReturn> for Vec<SourceRange> {
    fn from(item: ProgramReturn) -> Self {
        match item {
            ProgramReturn::Arguments(args) => args
                .iter()
                .map(|arg| {
                    let r: SourceRange = arg.into();
                    r
                })
                .collect(),
            ProgramReturn::Value(v) => v.into(),
        }
    }
}

impl ProgramReturn {
    pub fn get_value(&self) -> Result<MemoryItem, KclError> {
        match self {
            ProgramReturn::Value(v) => Ok(v.clone()),
            ProgramReturn::Arguments(args) => Err(KclError::Semantic(KclErrorDetails {
                message: format!("Cannot get value from arguments: {:?}", args),
                source_ranges: self.clone().into(),
            })),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ts_rs::TS, JsonSchema)]
#[ts(export)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum MemoryItem {
    UserVal {
        value: serde_json::Value,
        #[serde(rename = "__meta")]
        meta: Vec<Metadata>,
    },
    SketchGroup(SketchGroup),
    ExtrudeGroup(ExtrudeGroup),
    ExtrudeTransform(ExtrudeTransform),
    Function {
        #[serde(skip)]
        func: Option<MemoryFunction>,
        expression: Box<FunctionExpression>,
        #[serde(rename = "__meta")]
        meta: Vec<Metadata>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ts_rs::TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ExtrudeTransform {
    pub position: Position,
    pub rotation: Rotation,
    #[serde(rename = "__meta")]
    pub meta: Vec<Metadata>,
}

pub type MemoryFunction = fn(
    s: &[MemoryItem],
    memory: &ProgramMemory,
    expression: &FunctionExpression,
    metadata: &[Metadata],
    engine: &mut EngineConnection,
) -> Result<Option<ProgramReturn>, KclError>;

impl From<MemoryItem> for Vec<SourceRange> {
    fn from(item: MemoryItem) -> Self {
        match item {
            MemoryItem::UserVal { meta, .. } => meta.iter().map(|m| m.source_range).collect(),
            MemoryItem::SketchGroup(s) => s.meta.iter().map(|m| m.source_range).collect(),
            MemoryItem::ExtrudeGroup(e) => e.meta.iter().map(|m| m.source_range).collect(),
            MemoryItem::ExtrudeTransform(e) => e.meta.iter().map(|m| m.source_range).collect(),
            MemoryItem::Function { meta, .. } => meta.iter().map(|m| m.source_range).collect(),
        }
    }
}

impl MemoryItem {
    pub fn get_json_value(&self) -> Result<serde_json::Value, KclError> {
        if let MemoryItem::UserVal { value, .. } = self {
            Ok(value.clone())
        } else {
            Err(KclError::Semantic(KclErrorDetails {
                message: format!("Not a user value: {:?}", self),
                source_ranges: self.clone().into(),
            }))
        }
    }

    pub fn call_fn(
        &self,
        args: &[MemoryItem],
        memory: &ProgramMemory,
        engine: &mut EngineConnection,
    ) -> Result<Option<ProgramReturn>, KclError> {
        if let MemoryItem::Function { func, expression, meta } = self {
            if let Some(func) = func {
                func(args, memory, expression, meta, engine)
            } else {
                Err(KclError::Semantic(KclErrorDetails {
                    message: format!("Not a function: {:?}", self),
                    source_ranges: vec![],
                }))
            }
        } else {
            Err(KclError::Semantic(KclErrorDetails {
                message: format!("not a function: {:?}", self),
                source_ranges: vec![],
            }))
        }
    }
}

/// A sketch group is a collection of paths.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ts_rs::TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct SketchGroup {
    /// The id of the sketch group.
    pub id: uuid::Uuid,
    /// The paths in the sketch group.
    pub value: Vec<Path>,
    /// The starting path.
    pub start: BasePath,
    /// The position of the sketch group.
    pub position: Position,
    /// The rotation of the sketch group.
    pub rotation: Rotation,
    /// Metadata.
    #[serde(rename = "__meta")]
    pub meta: Vec<Metadata>,
}

impl SketchGroup {
    pub fn get_path_by_id(&self, id: &uuid::Uuid) -> Option<&Path> {
        self.value.iter().find(|p| p.get_id() == *id)
    }

    pub fn get_path_by_name(&self, name: &str) -> Option<&Path> {
        self.value.iter().find(|p| p.get_name() == name)
    }

    pub fn get_base_by_name_or_start(&self, name: &str) -> Option<&BasePath> {
        if self.start.name == name {
            Some(&self.start)
        } else {
            self.value.iter().find(|p| p.get_name() == name).map(|p| p.get_base())
        }
    }

    pub fn get_coords_from_paths(&self) -> Result<Point2d, KclError> {
        if self.value.is_empty() {
            return Ok(self.start.to.into());
        }

        let index = self.value.len() - 1;
        if let Some(path) = self.value.get(index) {
            let base = path.get_base();
            Ok(base.to.into())
        } else {
            Ok(self.start.to.into())
        }
    }
}

/// An extrude group is a collection of extrude surfaces.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ts_rs::TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct ExtrudeGroup {
    /// The id of the extrude group.
    pub id: uuid::Uuid,
    /// The extrude surfaces.
    pub value: Vec<ExtrudeSurface>,
    /// The height of the extrude group.
    pub height: f64,
    /// The position of the extrude group.
    pub position: Position,
    /// The rotation of the extrude group.
    pub rotation: Rotation,
    /// Metadata.
    #[serde(rename = "__meta")]
    pub meta: Vec<Metadata>,
}

impl ExtrudeGroup {
    pub fn get_path_by_id(&self, id: &uuid::Uuid) -> Option<&ExtrudeSurface> {
        self.value.iter().find(|p| p.get_id() == *id)
    }

    pub fn get_path_by_name(&self, name: &str) -> Option<&ExtrudeSurface> {
        self.value.iter().find(|p| p.get_name() == name)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ts_rs::TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub enum BodyType {
    Root,
    Sketch,
    Block,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Copy, Clone, ts_rs::TS, JsonSchema)]
#[ts(export)]
pub struct Position(pub [f64; 3]);

#[derive(Debug, Deserialize, Serialize, PartialEq, Copy, Clone, ts_rs::TS, JsonSchema)]
#[ts(export)]
pub struct Rotation(pub [f64; 4]);

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Copy, Clone, ts_rs::TS, JsonSchema)]
#[ts(export)]
pub struct SourceRange(pub [usize; 2]);

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, ts_rs::TS, JsonSchema)]
#[ts(export)]
pub struct Point2d {
    pub x: f64,
    pub y: f64,
}

impl From<[f64; 2]> for Point2d {
    fn from(p: [f64; 2]) -> Self {
        Self { x: p[0], y: p[1] }
    }
}

impl From<&[f64; 2]> for Point2d {
    fn from(p: &[f64; 2]) -> Self {
        Self { x: p[0], y: p[1] }
    }
}

impl From<Point2d> for [f64; 2] {
    fn from(p: Point2d) -> Self {
        [p.x, p.y]
    }
}

impl From<Point2d> for kittycad::types::Point2D {
    fn from(p: Point2d) -> Self {
        Self { x: p.x, y: p.y }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone, ts_rs::TS, JsonSchema)]
#[ts(export)]
pub struct Point3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Metadata.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ts_rs::TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// The source range.
    pub source_range: SourceRange,
}

impl From<SourceRange> for Metadata {
    fn from(source_range: SourceRange) -> Self {
        Self { source_range }
    }
}

/// A base path.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ts_rs::TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BasePath {
    /// The from point.
    pub from: [f64; 2],
    /// The to point.
    pub to: [f64; 2],
    /// The name of the path.
    pub name: String,
    /// Metadata.
    #[serde(rename = "__geoMeta")]
    pub geo_meta: GeoMeta,
}

/// Geometry metadata.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ts_rs::TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct GeoMeta {
    /// The id of the geometry.
    pub id: uuid::Uuid,
    /// Metadata.
    #[serde(flatten)]
    pub metadata: Metadata,
}

/// A path.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ts_rs::TS, JsonSchema)]
#[ts(export)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Path {
    /// A path that goes to a point.
    ToPoint {
        #[serde(flatten)]
        base: BasePath,
    },
    /// A path that is horizontal.
    Horizontal {
        #[serde(flatten)]
        base: BasePath,
        /// The x coordinate.
        x: f64,
    },
    /// An angled line to.
    AngledLineTo {
        #[serde(flatten)]
        base: BasePath,
        /// The x coordinate.
        x: Option<f64>,
        /// The y coordinate.
        y: Option<f64>,
    },
    /// A base path.
    Base {
        #[serde(flatten)]
        base: BasePath,
    },
}

impl Path {
    pub fn get_id(&self) -> uuid::Uuid {
        match self {
            Path::ToPoint { base } => base.geo_meta.id,
            Path::Horizontal { base, .. } => base.geo_meta.id,
            Path::AngledLineTo { base, .. } => base.geo_meta.id,
            Path::Base { base } => base.geo_meta.id,
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            Path::ToPoint { base } => base.name.clone(),
            Path::Horizontal { base, .. } => base.name.clone(),
            Path::AngledLineTo { base, .. } => base.name.clone(),
            Path::Base { base } => base.name.clone(),
        }
    }

    pub fn get_base(&self) -> &BasePath {
        match self {
            Path::ToPoint { base } => base,
            Path::Horizontal { base, .. } => base,
            Path::AngledLineTo { base, .. } => base,
            Path::Base { base } => base,
        }
    }
}

/// An extrude surface.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ts_rs::TS, JsonSchema)]
#[ts(export)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ExtrudeSurface {
    /// An extrude plane.
    ExtrudePlane {
        /// The position.
        position: Position,
        /// The rotation.
        rotation: Rotation,
        /// The name.
        name: String,
        /// Metadata.
        #[serde(flatten)]
        geo_meta: GeoMeta,
    },
}

impl ExtrudeSurface {
    pub fn get_id(&self) -> uuid::Uuid {
        match self {
            ExtrudeSurface::ExtrudePlane { geo_meta, .. } => geo_meta.id,
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            ExtrudeSurface::ExtrudePlane { name, .. } => name.clone(),
        }
    }

    pub fn get_position(&self) -> Position {
        match self {
            ExtrudeSurface::ExtrudePlane { position, .. } => *position,
        }
    }

    pub fn get_rotation(&self) -> Rotation {
        match self {
            ExtrudeSurface::ExtrudePlane { rotation, .. } => *rotation,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, ts_rs::TS, JsonSchema)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct PipeInfo {
    pub previous_results: Vec<MemoryItem>,
    pub is_in_pipe: bool,
    pub index: usize,
    pub body: Vec<Value>,
}

impl PipeInfo {
    pub fn new() -> Self {
        Self {
            previous_results: Vec::new(),
            is_in_pipe: false,
            index: 0,
            body: Vec::new(),
        }
    }
}

impl Default for PipeInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute a AST's program.
pub fn execute(
    program: crate::abstract_syntax_tree_types::Program,
    memory: &mut ProgramMemory,
    options: BodyType,
    engine: &mut EngineConnection,
) -> Result<ProgramMemory, KclError> {
    let mut pipe_info = PipeInfo::default();
    let stdlib = crate::std::StdLib::new();

    // Iterate over the body of the program.
    for statement in &program.body {
        match statement {
            BodyItem::ExpressionStatement(expression_statement) => {
                if let Value::CallExpression(call_expr) = &expression_statement.expression {
                    let fn_name = call_expr.callee.name.to_string();
                    let mut args: Vec<MemoryItem> = Vec::new();
                    for arg in &call_expr.arguments {
                        match arg {
                            Value::Literal(literal) => args.push(literal.into()),
                            Value::Identifier(identifier) => {
                                let memory_item = memory.get(&identifier.name, identifier.into())?;
                                args.push(memory_item.clone());
                            }
                            // We do nothing for the rest.
                            _ => (),
                        }
                    }
                    if fn_name == "show" {
                        if options != BodyType::Root {
                            return Err(KclError::Semantic(KclErrorDetails {
                                message: "Cannot call show outside of a root".to_string(),
                                source_ranges: vec![call_expr.into()],
                            }));
                        }

                        memory.return_ = Some(ProgramReturn::Arguments(call_expr.arguments.clone()));
                    } else if let Some(func) = memory.clone().root.get(&fn_name) {
                        func.call_fn(&args, memory, engine)?;
                    } else {
                        return Err(KclError::Semantic(KclErrorDetails {
                            message: format!("No such name {} defined", fn_name),
                            source_ranges: vec![call_expr.into()],
                        }));
                    }
                }
            }
            BodyItem::VariableDeclaration(variable_declaration) => {
                for declaration in &variable_declaration.declarations {
                    let var_name = declaration.id.name.to_string();
                    let source_range: SourceRange = declaration.init.clone().into();
                    let metadata = Metadata { source_range };

                    match &declaration.init {
                        Value::Literal(literal) => {
                            memory.add(&var_name, literal.into(), source_range)?;
                        }
                        Value::Identifier(identifier) => {
                            let value = memory.get(&identifier.name, identifier.into())?;
                            memory.add(&var_name, value.clone(), source_range)?;
                        }
                        Value::BinaryExpression(binary_expression) => {
                            let result = binary_expression.get_result(memory, &mut pipe_info, &stdlib, engine)?;
                            memory.add(&var_name, result, source_range)?;
                        }
                        Value::FunctionExpression(function_expression) => {
                            memory.add(
                                &var_name,
                                MemoryItem::Function{
                                    expression: function_expression.clone(),
                                    meta: vec![metadata],
                                    func: Some(|args: &[MemoryItem], memory: &ProgramMemory, function_expression: &FunctionExpression, _metadata: &[Metadata], engine: &mut EngineConnection| -> Result<Option<ProgramReturn>, KclError> {
                                        let mut fn_memory = memory.clone();

                                        if args.len() != function_expression.params.len() {
                                            return Err(KclError::Semantic(KclErrorDetails {
                                                message: format!("Expected {} arguments, got {}", function_expression.params.len(), args.len()),
                                                source_ranges: vec![function_expression.into()],
                                            }));
                                        }

                                        // Add the arguments to the memory.
                                        for (index, param) in function_expression.params.iter().enumerate() {
                                            fn_memory.add(
                                                &param.name,
                                                args.clone().get(index).unwrap().clone(),
                                                param.into(),
                                            )?;
                                        }

                                        let result = execute(function_expression.body.clone(), &mut fn_memory, BodyType::Block, engine)?;

                                        Ok(result.return_)
                                    })
                                },
                                source_range,
                            )?;
                        }
                        Value::CallExpression(call_expression) => {
                            let result = call_expression.execute(memory, &mut pipe_info, &stdlib, engine)?;
                            memory.add(&var_name, result, source_range)?;
                        }
                        Value::PipeExpression(pipe_expression) => {
                            let result = pipe_expression.get_result(memory, &mut pipe_info, &stdlib, engine)?;
                            memory.add(&var_name, result, source_range)?;
                        }
                        Value::PipeSubstitution(pipe_substitution) => {
                            return Err(KclError::Semantic(KclErrorDetails {
                                message: format!(
                                    "pipe substitution not implemented for declaration of variable {}",
                                    var_name
                                ),
                                source_ranges: vec![pipe_substitution.into()],
                            }));
                        }
                        Value::ArrayExpression(array_expression) => {
                            let result = array_expression.execute(memory, &mut pipe_info, &stdlib, engine)?;
                            memory.add(&var_name, result, source_range)?;
                        }
                        Value::ObjectExpression(object_expression) => {
                            let result = object_expression.execute(memory, &mut pipe_info, &stdlib, engine)?;
                            memory.add(&var_name, result, source_range)?;
                        }
                        Value::MemberExpression(member_expression) => {
                            let result = member_expression.get_result(memory)?;
                            memory.add(&var_name, result, source_range)?;
                        }
                        Value::UnaryExpression(unary_expression) => {
                            let result = unary_expression.get_result(memory, &mut pipe_info, &stdlib, engine)?;
                            memory.add(&var_name, result, source_range)?;
                        }
                    }
                }
            }
            BodyItem::ReturnStatement(return_statement) => match &return_statement.argument {
                Value::BinaryExpression(bin_expr) => {
                    let result = bin_expr.get_result(memory, &mut pipe_info, &stdlib, engine)?;
                    memory.return_ = Some(ProgramReturn::Value(result));
                }
                Value::Identifier(identifier) => {
                    let value = memory.get(&identifier.name, identifier.into())?.clone();
                    memory.return_ = Some(ProgramReturn::Value(value));
                }
                _ => (),
            },
        }
    }

    Ok(memory.clone())
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    pub async fn parse_execute(code: &str) -> Result<ProgramMemory> {
        let tokens = crate::tokeniser::lexer(code);
        let program = crate::parser::abstract_syntax_tree(&tokens)?;
        let mut mem: ProgramMemory = Default::default();
        let mut engine = EngineConnection::new().await?;
        let memory = execute(program, &mut mem, BodyType::Root, &mut engine)?;

        Ok(memory)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_execute_assign_two_variables() {
        let ast = r#"const myVar = 5
const newVar = myVar + 1"#;
        let memory = parse_execute(ast).await.unwrap();
        assert_eq!(
            serde_json::json!(5),
            memory.root.get("myVar").unwrap().get_json_value().unwrap()
        );
        assert_eq!(
            serde_json::json!(6.0),
            memory.root.get("newVar").unwrap().get_json_value().unwrap()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_execute_angled_line_that_intersects() {
        let ast_fn = |offset: &str| -> String {
            format!(
                r#"const part001 = startSketchAt([0, 0])
  |> lineTo({{to:[2, 2], tag: "yo"}}, %)
  |> lineTo([3, 1], %)
  |> angledLineThatIntersects({{
  angle: 180,
  intersectTag: 'yo',
  offset: {},
  tag: "yo2"
}}, %)
const intersect = segEndX('yo2', part001)
show(part001)"#,
                offset
            )
        };

        let memory = parse_execute(&ast_fn("-1")).await.unwrap();
        assert_eq!(
            serde_json::json!(1.0 + 2.0f64.sqrt()),
            memory.root.get("intersect").unwrap().get_json_value().unwrap()
        );

        let memory = parse_execute(&ast_fn("0")).await.unwrap();
        assert_eq!(
            serde_json::json!(1.0000000000000002),
            memory.root.get("intersect").unwrap().get_json_value().unwrap()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_execute_fn_definitions() {
        let ast = r#"const def = (x) => {
  return x
}
const ghi = (x) => {
  return x
}
const jkl = (x) => {
  return x
}
const hmm = (x) => {
  return x
}

const yo = 5 + 6

const abc = 3
const identifierGuy = 5
const part001 = startSketchAt([-1.2, 4.83])
|> line([2.8, 0], %)
|> angledLine([100 + 100, 3.01], %)
|> angledLine([abc, 3.02], %)
|> angledLine([def(yo), 3.03], %)
|> angledLine([ghi(2), 3.04], %)
|> angledLine([jkl(yo) + 2, 3.05], %)
|> close(%)
const yo2 = hmm([identifierGuy + 5])
show(part001)"#;

        parse_execute(ast).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_execute_with_pipe_substitutions_unary() {
        let ast = r#"const myVar = 3
const part001 = startSketchAt([0, 0])
  |> line({ to: [3, 4], tag: 'seg01' }, %)
  |> line([
  min(segLen('seg01', %), myVar),
  -legLen(segLen('seg01', %), myVar)
], %)

show(part001)"#;

        parse_execute(ast).await.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_execute_with_pipe_substitutions() {
        let ast = r#"const myVar = 3
const part001 = startSketchAt([0, 0])
  |> line({ to: [3, 4], tag: 'seg01' }, %)
  |> line([
  min(segLen('seg01', %), myVar),
  legLen(segLen('seg01', %), myVar)
], %)

show(part001)"#;

        parse_execute(ast).await.unwrap();
    }
}