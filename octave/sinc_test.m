N = 100;
P = 10;
t = -N:N;
s = sinc(P*t/N);

figure(1)
clf
plot(t,s)
hold on
grid on



