using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000A1 RID: 161
	[HandlerCategory("vvPosClose"), HandlerName("ATR trail")]
	public class TrailStopOnATR : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x060005D6 RID: 1494 RVA: 0x0001C57C File Offset: 0x0001A77C
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			ISecurity security = pos.get_Security();
			double num = pos.OpenMFEPct(barNum);
			double num6;
			if (num > this.TrailEnable)
			{
				IList<double> highPrices = security.get_HighPrices();
				IList<double> lowPrices = security.get_LowPrices();
				IList<double> arg_3D_0 = security.get_ClosePrices();
				IList<double> arg_44_0 = security.get_OpenPrices();
				double num2 = TrailStop.CalcATR(security, this.ATRperiod, barNum, 0);
				double num3 = lowPrices[pos.FindLowBar(barNum)];
				double num4 = highPrices[pos.FindHighBar(barNum)];
				double num5 = num2 * this.ATRcoef;
				num6 = (pos.get_IsLong() ? (num4 - num5) : (num3 + num5));
			}
			else
			{
				double num5 = (0.0 - this.StopLoss) / 100.0;
				num6 = pos.get_EntryPrice() * (1.0 + (pos.get_IsLong() ? num5 : (-num5)));
			}
			double stop = pos.GetStop(barNum);
			if (stop != 0.0)
			{
				num6 = (pos.get_IsLong() ? Math.Max(num6, stop) : Math.Min(num6, stop));
			}
			return num6;
		}

		// Token: 0x170001FD RID: 509
		[HandlerParameter(true, "2", Min = "1", Max = "10", Step = "1", Name = "Коэф. ATR")]
		public double ATRcoef
		{
			// Token: 0x060005D2 RID: 1490 RVA: 0x0001C557 File Offset: 0x0001A757
			get;
			// Token: 0x060005D3 RID: 1491 RVA: 0x0001C55F File Offset: 0x0001A75F
			set;
		}

		// Token: 0x170001FE RID: 510
		[HandlerParameter(true, "5", Min = "1", Max = "10", Step = "1", Name = "Период расч. ATR")]
		public int ATRperiod
		{
			// Token: 0x060005D4 RID: 1492 RVA: 0x0001C568 File Offset: 0x0001A768
			get;
			// Token: 0x060005D5 RID: 1493 RVA: 0x0001C570 File Offset: 0x0001A770
			set;
		}

		// Token: 0x170001FB RID: 507
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "0.7", Step = "0.1", Name = "Stop Loss (в %)")]
		public double StopLoss
		{
			// Token: 0x060005CE RID: 1486 RVA: 0x0001C535 File Offset: 0x0001A735
			get;
			// Token: 0x060005CF RID: 1487 RVA: 0x0001C53D File Offset: 0x0001A73D
			set;
		}

		// Token: 0x170001FC RID: 508
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "0.7", Step = "0.1", Name = "Trail Enable (в %)")]
		public double TrailEnable
		{
			// Token: 0x060005D0 RID: 1488 RVA: 0x0001C546 File Offset: 0x0001A746
			get;
			// Token: 0x060005D1 RID: 1489 RVA: 0x0001C54E File Offset: 0x0001A74E
			set;
		}
	}
}
