use crate::{
    buffer::Buffer,
    layout::{Rect, Size},
    style::Style,
    symbols::{self},
    widgets::{Block, Widget},
};
use std::cmp::min;
use unicode_width::UnicodeWidthStr;

use super::SizeHint;

/// Display multiple bars in a single widgets
///
/// # Examples
///
/// ```
/// # use ratatui::widgets::{Block, Borders, BarChart};
/// # use ratatui::style::{Style, Color, Modifier};
/// BarChart2::default()
///     .block(Block::default().title("BarChart2").borders(Borders::ALL))
///     .bar_width(3)
///     .bar_gap(1)
///     .bar_style(Style::default().fg(Color::Yellow).bg(Color::Red))
///     .value_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
///     .label_style(Style::default().fg(Color::White))
///     .data(&[("B0", 0), ("B1", 2), ("B2", 4), ("B3", 3)])
///     .max(4);
/// ```
#[derive(Debug, Clone)]
pub struct BarChart2<'a> {
    /// Block to wrap the widget in
    block: Option<Block<'a>>,
    /// The width of each bar
    bar_width: u16,
    /// The gap between each bar
    bar_gap: u16,
    group_gap: u16,
    /// Set of symbols used to display the data
    bar_set: symbols::bar::Set,
    /// Style of the bars
    bar_styles: &'a [Style],
    /// Style of the values printed at the bottom of each bar
    value_styles: &'a [Style],
    /// Style of the labels printed under each bar
    label_style: Style,
    /// Style for the widget
    style: Style,
    /// Slice of value pair to plot on the chart
    data: Vec<Vec<u64>>,

    labels: &'a [&'a str],
    /// Value necessary for a bar to reach the maximum height (if no value is specified,
    /// the maximum value in the data is taken as reference)
    max: Option<u64>,
    /// Values to display on the bar (computed when the data is passed to the widget)
    format: fn(u64) -> String,
}

fn format_value(value: u64) -> String {
    value.to_string()
}

impl<'a> Default for BarChart2<'a> {
    fn default() -> BarChart2<'a> {
        BarChart2 {
            block: None,
            max: None,
            data: Vec::new(),
            labels: &[],
            bar_styles: &[],
            bar_width: 1,
            bar_gap: 1,
            group_gap: 1,
            bar_set: symbols::bar::NINE_LEVELS,
            value_styles: &[],
            label_style: Style::default(),
            style: Style::default(),
            format: format_value,
        }
    }
}

impl<'a> BarChart2<'a> {
    pub fn add_data(mut self, data: &[u64]) -> BarChart2<'a> {
        if self.data.is_empty() {
            self.data = data.iter().map(|&v| vec![v]).collect();
        } else {
            for (list, &e) in self.data.iter_mut().zip(data) {
                list.push(e);
            }
        }
        self
    }

    pub fn block(mut self, block: Block<'a>) -> BarChart2<'a> {
        self.block = Some(block);
        self
    }

    pub fn value_format(mut self, value_format: fn(u64) -> String) -> BarChart2<'a> {
        self.format = value_format;
        self
    }

    pub fn max(mut self, max: u64) -> BarChart2<'a> {
        self.max = Some(max);
        self
    }

    pub fn bar_styles(mut self, styles: &'a [Style]) -> BarChart2<'a> {
        self.bar_styles = styles;
        self
    }

    pub fn bar_width(mut self, width: u16) -> BarChart2<'a> {
        self.bar_width = width;
        self
    }

    pub fn bar_gap(mut self, gap: u16) -> BarChart2<'a> {
        self.bar_gap = gap;
        self
    }

    pub fn group_gap(mut self, gap: u16) -> BarChart2<'a> {
        self.group_gap = gap;
        self
    }

    pub fn bar_set(mut self, bar_set: symbols::bar::Set) -> BarChart2<'a> {
        self.bar_set = bar_set;
        self
    }

    pub fn value_styles(mut self, styles: &'a [Style]) -> BarChart2<'a> {
        self.value_styles = styles;
        self
    }

    pub fn label_style(mut self, style: Style) -> BarChart2<'a> {
        self.label_style = style;
        self
    }

    pub fn style(mut self, style: Style) -> BarChart2<'a> {
        self.style = style;
        self
    }

    pub fn labels(mut self, labels: &'a [&'a str]) -> BarChart2<'a> {
        self.labels = labels;
        self
    }
}

impl<'a> Widget for BarChart2<'a> {
    fn render(mut self, area: Rect, buf: &mut Buffer) {
        buf.set_style(area, self.style);

        let chart_area = match self.block.take() {
            Some(b) => {
                let inner_area = b.inner(area);
                b.render(area, buf);
                inner_area
            }
            None => area,
        };

        if chart_area.height < 2 || self.data.is_empty() {
            return;
        }

        let max = self.max.unwrap_or_else(|| {
            self.data
                .iter()
                .map(|t| t.iter().max().copied().unwrap_or_default())
                .max()
                .unwrap_or_default()
        });

        let bars_per_column = self.data[0].len();

        let max_index = min(
            (chart_area.width + self.group_gap + self.bar_gap) as usize
                / ((self.bar_width + self.bar_gap) * bars_per_column as u16 + self.group_gap)
                    as usize,
            self.data.len(),
        );

        let mut data: Vec<Vec<u64>> = self
            .data
            .iter()
            .take(max_index)
            .map(|bars| {
                bars.iter()
                    .map(|v| v * u64::from(chart_area.height - 1) * 8 / std::cmp::max(max, 1))
                    .collect()
            })
            .collect::<Vec<Vec<u64>>>();

        let defaul_style = Style::default();
        for j in (0..chart_area.height - 1).rev() {
            let mut i = 0usize;
            let mut x_offset = 0u16;
            for d in data.iter_mut() {
                for (data_type, d) in d.iter_mut().enumerate() {
                    let symbol = match d {
                        0 => self.bar_set.empty,
                        1 => self.bar_set.one_eighth,
                        2 => self.bar_set.one_quarter,
                        3 => self.bar_set.three_eighths,
                        4 => self.bar_set.half,
                        5 => self.bar_set.five_eighths,
                        6 => self.bar_set.three_quarters,
                        7 => self.bar_set.seven_eighths,
                        _ => self.bar_set.full,
                    };

                    let bar_style = self.bar_styles.get(data_type).unwrap_or(&defaul_style);

                    for x in 0..self.bar_width {
                        buf.get_mut(
                            chart_area.left()
                                + i as u16 * (self.bar_width + self.bar_gap)
                                + x
                                + x_offset,
                            chart_area.top() + j,
                        )
                        .set_symbol(symbol)
                        .set_style(*bar_style);
                    }

                    i += 1;
                    if *d > 8 {
                        *d -= 8;
                    } else {
                        *d = 0;
                    }
                }
                x_offset += self.group_gap;
            }
        }

        let mut i = max_index * bars_per_column;
        let mut x_offset = self.group_gap * max_index as u16;

        for d in self.data.into_iter().take(max_index).rev() {
            x_offset -= self.group_gap;
            for (data_type, value) in d.into_iter().enumerate().rev() {
                i -= 1;
                if value != 0 {
                    let value_label = (self.format)(value);
                    let width = value_label.width() as u16;
                    let style = self.value_styles.get(data_type).unwrap_or(&defaul_style);
                    buf.set_string(
                        chart_area.left()
                            + i as u16 * (self.bar_width + self.bar_gap)
                            + x_offset
                            + (self.bar_width.saturating_sub(width) >> 1),
                        chart_area.bottom() - 2 - data_type as u16,
                        value_label,
                        *style,
                    );
                }
            }
        }

        let label_max_width =
            bars_per_column as u16 * self.bar_width + (bars_per_column as u16 - 1) * self.bar_gap;
        for (i, label) in self.labels.iter().take(max_index).enumerate() {
            buf.set_stringn(
                chart_area.left()
                    + (i * bars_per_column) as u16 * (self.bar_width + self.bar_gap)
                    + (self.group_gap * i as u16)
                    + (label_max_width.saturating_sub(label.len() as u16) >> 1),
                chart_area.bottom() - 1,
                label,
                label_max_width as usize,
                self.label_style,
            );
        }
    }
}

impl<'a> SizeHint for BarChart2<'a> {
    fn size_hint(&self, area: &Rect) -> Size {
        match &self.block {
            Some(b) => {
                let block_area = b.size_hint(&Rect::default());
                Size::new(
                    area.width.max(block_area.width),
                    area.height.max(block_area.height + 1),
                )
            }
            None => Size::new(area.width, area.height),
        }
    }
}
