SF = 48000;
F = 2;
T = 1;
RATIO = 0.5;

t_in = 0:SF * T;
in = sin(t_in * 2 * pi * F / SF);

for k = 1:length(in) / RATIO
    out(k) = in(floor((k - 1) * RATIO) + 1) * 2;
end

figure(1)
clf
plot(in)
hold on
plot(out)
grid on
