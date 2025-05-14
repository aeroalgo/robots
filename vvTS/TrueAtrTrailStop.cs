using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020000A5 RID: 165
	[HandlerCategory("vvPosClose"), HandlerName("True ATR Trail")]
	public class TrueAtrTrailStop : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x060005F8 RID: 1528 RVA: 0x0001CB08 File Offset: 0x0001AD08
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			ISecurity security = pos.get_Security();
			double num = 0.0;
			double num2 = 0.0;
			bool flag = false;
			IList<double> highPrices = security.get_HighPrices();
			IList<double> lowPrices = security.get_LowPrices();
			IList<double> arg_4C_0 = security.get_ClosePrices();
			IList<double> arg_53_0 = security.get_OpenPrices();
			double num3 = TrailStop.CalcATR(security, this.ATRperiod, pos.get_EntryBarNum(), 0);
			if (pos.get_IsLong())
			{
				num = highPrices[pos.FindHighBar(barNum)];
				if (num > pos.get_EntryPrice() + num3 * this.TrailEnable)
				{
					flag = true;
				}
			}
			else
			{
				num2 = lowPrices[pos.FindLowBar(barNum)];
				if (num2 < pos.get_EntryPrice() - num3 * this.TrailEnable)
				{
					flag = true;
				}
			}
			double num6;
			if (flag)
			{
				double num4 = TrailStop.CalcATR(security, this.ATRperiod, barNum, 0);
				double num5 = num4 * this.ATRcoef;
				num6 = (pos.get_IsLong() ? (num - num5) : (num2 + num5));
			}
			else
			{
				num6 = (pos.get_IsLong() ? (pos.get_EntryPrice() - num3 * this.StopLoss) : (pos.get_EntryPrice() + num3 * this.StopLoss));
			}
			double stop = pos.GetStop(barNum);
			if (stop != 0.0)
			{
				num6 = (pos.get_IsLong() ? Math.Max(num6, stop) : Math.Min(num6, stop));
			}
			return num6;
		}

		// Token: 0x1700020A RID: 522
		[HandlerParameter(true, "2", Min = "1", Max = "10", Step = "0.2", Name = "Размер трейла(в ATR)")]
		public double ATRcoef
		{
			// Token: 0x060005F4 RID: 1524 RVA: 0x0001CAE3 File Offset: 0x0001ACE3
			get;
			// Token: 0x060005F5 RID: 1525 RVA: 0x0001CAEB File Offset: 0x0001ACEB
			set;
		}

		// Token: 0x1700020B RID: 523
		[HandlerParameter(true, "10", Min = "2", Max = "10", Step = "1", Name = "Период расч. ATR")]
		public int ATRperiod
		{
			// Token: 0x060005F6 RID: 1526 RVA: 0x0001CAF4 File Offset: 0x0001ACF4
			get;
			// Token: 0x060005F7 RID: 1527 RVA: 0x0001CAFC File Offset: 0x0001ACFC
			set;
		}

		// Token: 0x17000208 RID: 520
		[HandlerParameter(true, "1.5", Min = "1", Max = "5", Step = "0.1", Name = "Стоп-лосс(в ATR)")]
		public double StopLoss
		{
			// Token: 0x060005F0 RID: 1520 RVA: 0x0001CAC1 File Offset: 0x0001ACC1
			get;
			// Token: 0x060005F1 RID: 1521 RVA: 0x0001CAC9 File Offset: 0x0001ACC9
			set;
		}

		// Token: 0x17000209 RID: 521
		[HandlerParameter(true, "1", Min = "1", Max = "5", Step = "0.1", Name = "Включить трейл(в ATR)")]
		public double TrailEnable
		{
			// Token: 0x060005F2 RID: 1522 RVA: 0x0001CAD2 File Offset: 0x0001ACD2
			get;
			// Token: 0x060005F3 RID: 1523 RVA: 0x0001CADA File Offset: 0x0001ACDA
			set;
		}
	}
}
