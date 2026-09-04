use leptos::prelude::*;

#[component]
pub fn Droplet() -> impl IntoView {
    view! {
        <style>
            "
            .droplet-container {
                position: absolute;
                inset: 0;
                pointer-events: none;
                overflow: hidden;
            }
            .droplet {
                position: absolute;
                top: 6%;
                right: 10%;
                width: 14px;
                height: 20px;
                border-radius: 50% 50% 50% 50% / 60% 60% 40% 40%;
                background: radial-gradient(
                    circle at 35% 30%,
                    rgba(255, 255, 255, 0.9) 0%,
                    rgba(76, 168, 189, 0.6) 50%,
                    rgba(76, 168, 189, 0.2) 100%
                );
                box-shadow:
                    0 0 8px rgba(76, 168, 189, 0.6),
                    0 -18px 12px -8px rgba(76, 168, 189, 0.4);
                opacity: 0;
                animation: droplet-fall 3s ease-in forwards;
            }
            .droplet-residue {
                position: absolute;
                bottom: 8%;
                right: 10%;
                width: 20px;
                height: 10px;
                border-radius: 50%;
                background: radial-gradient(circle, rgba(76,168,189,0.3) 0%, transparent 70%);
                opacity: 0;
                animation: residue-fade 1s ease-out 2.8s forwards;
            }
            @keyframes droplet-fall {
                0%   { top: 6%;  opacity: 0; transform: translateX(0) rotate(0deg) scale(0.8); }
                10%  { opacity: 1; transform: translateX(0) rotate(0deg) scale(1); }
                25%  { transform: translateX(-4px) rotate(-3deg); }
                50%  { transform: translateX(4px) rotate(3deg); }
                75%  { transform: translateX(-3px) rotate(-2deg); }
                100% { top: 92%; opacity: 0; transform: translateX(0) rotate(0deg) scale(0.9); }
            }
            @keyframes residue-fade {
                0%   { opacity: 0; transform: scale(0.5); }
                50%  { opacity: 1; transform: scale(1); }
                100% { opacity: 0; transform: scale(1.2); }
            }
            "
        </style>
        <div class="droplet-container">
            <div class="droplet"></div>
            <div class="droplet-residue"></div>
        </div>
    }
}
