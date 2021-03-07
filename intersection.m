supply_shock = input('supply_shock = ');
subsidies = input('subsidies = ');
trending = input('trending = ');
y_p = supply_shock-subsidies;
y_c = trending;
function [err] = calc(t, y_p, y_c)
	xcoord_p = 3*(1-t(1))^2*t(1)*10 + 3*(1-t(1))*t(1)^2*50 + t(1)^3*80;
	ycoord_p = (1-t(1))^3*(80+y_p) + 3*(1-t(1))^2*t(1)*(-10+y_p) + 3*(1-t(1))*t(1)^2*(-10+y_p) + t(1)^3*(100+y_p);
	xcoord_c = 3*(1-t(2))^2*t(2)*40 + 3*(1-t(2))*t(2)^2*70 + t(2)^3*80;
	ycoord_c = (1-t(2))^3*(80+y_c) + 3*(1-t(2))^2*t(2)*(80+y_c) + 3*(1-t(2))*t(2)^2*(70+y_c) + t(2)^3*(y_c);
	err = (xcoord_p-xcoord_c)^2+(ycoord_p-ycoord_c)^2;
	% prevent the algorithm from staying at t = 0
	if xcoord_p < 0.5
		err = 10000-xcoord_p;
	endif
endfunction
f = @(t) calc(t,y_p,y_c); 
t = fsolve(f, [0,0]);

disp(['t_p = ', num2str(t(1))])
disp(['t_c = ', num2str(t(2))])
disp(['x = ', num2str(3*(1-t(1))^2*t(1)*10 + 3*(1-t(1))*t(1)^2*50 + t(1)^3*80)])
disp(['y = ', num2str((1-t(1))^3*(80+y_p) + 3*(1-t(1))^2*t(1)*(-10+y_p) + 3*(1-t(1))*t(1)^2*(-10+y_p) + t(1)^3*(100+y_p))])
