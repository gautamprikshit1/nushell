use crate::{commands::dataframe::utils::parse_polars_error, prelude::*};
use nu_engine::WholeStreamCommand;
use nu_errors::ShellError;
use nu_protocol::{
    dataframe::{NuDataFrame, PolarsData},
    Signature, SyntaxShape, UntaggedValue, Value,
};
use nu_source::Tagged;

use polars::prelude::DataType;

enum Operation {
    First,
    Sum,
    Min,
    Max,
    Mean,
    Median,
}

impl Operation {
    fn from_tagged(name: &Tagged<String>) -> Result<Operation, ShellError> {
        match name.item.as_ref() {
            "first" => Ok(Operation::First),
            "sum" => Ok(Operation::Sum),
            "min" => Ok(Operation::Min),
            "max" => Ok(Operation::Max),
            "mean" => Ok(Operation::Mean),
            "median" => Ok(Operation::Median),
            _ => Err(ShellError::labeled_error_with_secondary(
                "Operation not fount",
                "Operation does not exist for pivot",
                &name.tag,
                "Perhaps you want: first, sum, min, max, mean, median",
                &name.tag,
            )),
        }
    }
}

pub struct DataFrame;

impl WholeStreamCommand for DataFrame {
    fn name(&self) -> &str {
        "pls pivot"
    }

    fn usage(&self) -> &str {
        "Performs a pivot operation on a groupby object"
    }

    fn signature(&self) -> Signature {
        Signature::build("pls pivot")
            .required(
                "pivot column",
                SyntaxShape::String,
                "pivot column to perform pivot",
            )
            .required(
                "value column",
                SyntaxShape::String,
                "value column to perform pivot",
            )
            .required("operation", SyntaxShape::String, "aggregate operation")
    }

    fn run(&self, args: CommandArgs) -> Result<OutputStream, ShellError> {
        command(args)
    }

    fn examples(&self) -> Vec<Example> {
        vec![Example {
            description: "Pivot a dataframe on b and aggregation on col c",
            example:
                "[[a b c]; [one x 1] [two y 2]] | pls convert | pls groupby [a] | pls pivot b c sum",
            result: None,
        }]
    }
}

fn command(args: CommandArgs) -> Result<OutputStream, ShellError> {
    let tag = args.call_info.name_tag.clone();
    let mut args = args.evaluate_once()?;

    // Extracting the pivot col from arguments
    let pivot_col: Tagged<String> = args.req(0)?;

    // Extracting the value col from arguments
    let value_col: Tagged<String> = args.req(1)?;

    let operation: Tagged<String> = args.req(2)?;
    let op = Operation::from_tagged(&operation)?;

    // The operation is only done in one groupby. Only one input is
    // expected from the InputStream
    match args.input.next() {
        None => Err(ShellError::labeled_error(
            "No input received",
            "missing groupby input from stream",
            &tag,
        )),
        Some(value) => {
            if let UntaggedValue::DataFrame(PolarsData::GroupBy(nu_groupby)) = value.value {
                let df_ref = nu_groupby.as_ref();

                check_pivot_column(df_ref, &pivot_col)?;
                check_value_column(df_ref, &value_col)?;

                let mut groupby = nu_groupby.to_groupby()?;

                let pivot = groupby.pivot(pivot_col.item.as_ref(), value_col.item.as_ref());

                let res = match op {
                    Operation::Mean => pivot.mean(),
                    Operation::Sum => pivot.sum(),
                    Operation::Min => pivot.min(),
                    Operation::Max => pivot.max(),
                    Operation::First => pivot.first(),
                    Operation::Median => pivot.median(),
                }
                .map_err(|e| parse_polars_error::<&str>(&e, &tag.span, None))?;

                let final_df = Value {
                    tag,
                    value: UntaggedValue::DataFrame(PolarsData::EagerDataFrame(NuDataFrame::new(
                        res,
                    ))),
                };

                Ok(OutputStream::one(final_df))
            } else {
                Err(ShellError::labeled_error(
                    "No groupby in stream",
                    "no groupby found in input stream",
                    &tag,
                ))
            }
        }
    }
}

fn check_pivot_column(
    df: &polars::prelude::DataFrame,
    col: &Tagged<String>,
) -> Result<(), ShellError> {
    let series = df
        .column(col.item.as_ref())
        .map_err(|e| parse_polars_error::<&str>(&e, &col.tag.span, None))?;

    match series.dtype() {
        DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64
        | DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::Utf8 => Ok(()),
        _ => Err(ShellError::labeled_error(
            "Pivot error",
            format!("Unsupported datatype {}", series.dtype()),
            col.tag.span,
        )),
    }
}

fn check_value_column(
    df: &polars::prelude::DataFrame,
    col: &Tagged<String>,
) -> Result<(), ShellError> {
    let series = df
        .column(col.item.as_ref())
        .map_err(|e| parse_polars_error::<&str>(&e, &col.tag.span, None))?;

    match series.dtype() {
        DataType::UInt8
        | DataType::UInt16
        | DataType::UInt32
        | DataType::UInt64
        | DataType::Int8
        | DataType::Int16
        | DataType::Int32
        | DataType::Int64
        | DataType::Float32
        | DataType::Float64 => Ok(()),
        _ => Err(ShellError::labeled_error(
            "Pivot error",
            format!("Unsupported datatype {}", series.dtype()),
            col.tag.span,
        )),
    }
}
