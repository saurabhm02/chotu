use gloo_timers::callback::Interval;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct LatticeLoaderProps {
    #[prop_or("Thinking".to_string())]
    pub label: String,
    #[prop_or(true)]
    pub show_timer: bool,
    #[prop_or(3.5)]
    pub cell_size: f64,
    #[prop_or(1.5)]
    pub gap: f64,
    #[prop_or(11.0)]
    pub font_size: f64,
}

#[function_component(LatticeLoader)]
pub fn lattice_loader(props: &LatticeLoaderProps) -> Html {
    let elapsed_tenths = use_state(|| 0u32);

    {
        let elapsed_tenths = elapsed_tenths.clone();
        use_effect_with((), move |_| {
            let handle = Interval::new(100, move || {
                elapsed_tenths.set(*elapsed_tenths + 1);
            });
            move || drop(handle)
        });
    }

    // 3x3 orbit pattern:
    // [0, 1, 2]
    // [7, None, 3]
    // [6, 5, 4]
    let cells: [Option<u32>; 9] = [
        Some(0),
        Some(1),
        Some(2),
        Some(7),
        None,
        Some(3),
        Some(6),
        Some(5),
        Some(4),
    ];

    let step = 90.0;
    let scale = 1.2;
    let d = step * scale; // 108ms
    let cycle = 8.0 * d; // 864ms

    let tenths = *elapsed_tenths;
    let seconds = (tenths as f64) / 10.0;
    let timer_str = format!("{:.1}s", seconds);

    html! {
        <div class="inline-flex items-center gap-2 text-white select-none px-2.5 py-1 bg-[#22252c] rounded-md border border-white/10">
            <div
                class="grid grid-cols-3 shrink-0"
                style={format!(
                    "gap: {}px; width: {}px; height: {}px;",
                    props.gap,
                    props.cell_size * 3.0 + props.gap * 2.0,
                    props.cell_size * 3.0 + props.gap * 2.0
                )}
            >
                { for cells.iter().enumerate().map(|(i, &unit)| {
                    let style = match unit {
                        Some(u) => {
                            let delay = (u as f64) * d;
                            format!(
                                "width: {}px; height: {}px; background-color: #f5f5f5; border-radius: 9999px; animation: lattice-on {}ms cubic-bezier(0.77, 0, 0.175, 1) infinite; animation-delay: {}ms;",
                                props.cell_size, props.cell_size, cycle, delay
                            )
                        }
                        None => {
                            format!(
                                "width: {}px; height: {}px; background-color: #f5f5f5; border-radius: 9999px; opacity: 0.07;",
                                props.cell_size, props.cell_size
                            )
                        }
                    };
                    html! {
                        <span key={i} style={style}></span>
                    }
                }) }
            </div>

            <span class="font-medium text-[11px] text-gray-300 tracking-wide">
                {&props.label}
            </span>

            if props.show_timer {
                <span class="font-mono text-[11px] tabular-nums text-gray-400">
                    {timer_str}
                </span>
            }
        </div>
    }
}
