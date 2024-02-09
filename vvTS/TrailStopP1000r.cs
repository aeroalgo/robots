using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000AA RID: 170
	[HandlerCategory("vvPosClose"), HandlerName("TrailStop P1000R")]
	public class TrailStopP1000r : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600063B RID: 1595 RVA: 0x0001D4AE File Offset: 0x0001B6AE
		public double Execute(IPosition pos, int barNum)
		{
			return TrailStopP1000r.GenTrailStopP1000r(pos, barNum, this.StopLoss, this.TrailEnable, this.TrailLoss, this.ProfitLimit, this.TrailLoss2);
		}

		// Token: 0x0600063A RID: 1594 RVA: 0x0001D3C4 File Offset: 0x0001B5C4
		public static double GenTrailStopP1000r(IPosition _pos, int _barNum, double _StopLoss, double _TrailEnable, double _TrailLoss, double _ProfitLimit, double _TrailLoss2)
		{
			if (_pos == null)
			{
				return 0.0;
			}
			double num = _pos.OpenMFEPct(_barNum);
			double num2 = (0.0 - _StopLoss) / 100.0;
			double num3 = _pos.get_EntryPrice() * (1.0 + (_pos.get_IsLong() ? num2 : (-num2)));
			double val = num3;
			if (num > _TrailEnable)
			{
				num2 = ((num > _ProfitLimit) ? ((num - _TrailLoss2) / 100.0) : ((num - _TrailLoss) / 100.0));
				num3 = _pos.get_EntryPrice() * (1.0 + (_pos.get_IsLong() ? num2 : (-num2)));
			}
			num3 = (_pos.get_IsLong() ? Math.Max(num3, val) : Math.Min(num3, val));
			double stop = _pos.GetStop(_barNum);
			if (stop == 0.0)
			{
				return num3;
			}
			if (!_pos.get_IsLong())
			{
				return Math.Min(num3, stop);
			}
			return Math.Max(num3, stop);
		}

		// Token: 0x17000226 RID: 550
		[HandlerParameter(true, "0", Min = "0", Max = "0.6", Step = "0.05", Name = "ProfitLimit")]
		public double ProfitLimit
		{
			// Token: 0x06000636 RID: 1590 RVA: 0x0001D3A0 File Offset: 0x0001B5A0
			get;
			// Token: 0x06000637 RID: 1591 RVA: 0x0001D3A8 File Offset: 0x0001B5A8
			set;
		}

		// Token: 0x17000223 RID: 547
		[HandlerParameter(true, "1.5", Min = "0.1", Max = "0.6", Step = "0.05", Name = "StopLoss")]
		public double StopLoss
		{
			// Token: 0x06000630 RID: 1584 RVA: 0x0001D36D File Offset: 0x0001B56D
			get;
			// Token: 0x06000631 RID: 1585 RVA: 0x0001D375 File Offset: 0x0001B575
			set;
		}

		// Token: 0x17000224 RID: 548
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "0.6", Step = "0.05", Name = "TrailEnable")]
		public double TrailEnable
		{
			// Token: 0x06000632 RID: 1586 RVA: 0x0001D37E File Offset: 0x0001B57E
			get;
			// Token: 0x06000633 RID: 1587 RVA: 0x0001D386 File Offset: 0x0001B586
			set;
		}

		// Token: 0x17000225 RID: 549
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "0.6", Step = "0.05", Name = "TrailLoss")]
		public double TrailLoss
		{
			// Token: 0x06000634 RID: 1588 RVA: 0x0001D38F File Offset: 0x0001B58F
			get;
			// Token: 0x06000635 RID: 1589 RVA: 0x0001D397 File Offset: 0x0001B597
			set;
		}

		// Token: 0x17000227 RID: 551
		[HandlerParameter(true, "0", Min = "0", Max = "0.6", Step = "0.05", Name = "TrLoss2")]
		public double TrailLoss2
		{
			// Token: 0x06000638 RID: 1592 RVA: 0x0001D3B1 File Offset: 0x0001B5B1
			get;
			// Token: 0x06000639 RID: 1593 RVA: 0x0001D3B9 File Offset: 0x0001B5B9
			set;
		}
	}
}
