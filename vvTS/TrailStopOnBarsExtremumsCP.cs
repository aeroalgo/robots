using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200009B RID: 155
	[HandlerCategory("vvPosClose"), HandlerName("Трейл по теням CP Pct")]
	public class TrailStopOnBarsExtremumsCP : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x0600059C RID: 1436 RVA: 0x0001BC64 File Offset: 0x00019E64
		public double Execute(IPosition pos, int barNum)
		{
			if (pos == null)
			{
				return 0.0;
			}
			double num = pos.OpenMFE(barNum);
			double num2 = pos.get_EntryPrice();
			if (this.UseCalcPrice)
			{
				double num3 = CalcEntryPrice.EntryPrice(pos, barNum);
				double num4 = (num2 - num3) * (double)(pos.get_IsLong() ? 1 : -1);
				num += num4;
				num2 = num3;
			}
			num *= 100.0 / num2 * pos.get_Security().get_Margin();
			double num5 = (0.0 - this.StopLoss) / 100.0;
			double num6 = num2 * (1.0 + (pos.get_IsLong() ? num5 : (-num5)));
			double val = num6;
			if (num > this.TrailEnable)
			{
				ISecurity security = pos.get_Security();
				IList<double> highPrices = security.get_HighPrices();
				IList<double> lowPrices = security.get_LowPrices();
				IList<Bar> arg_CD_0 = security.get_Bars();
				int num7 = this.BarsCount;
				if (num7 > barNum)
				{
					num7 = barNum;
				}
				double num8 = highPrices[barNum - num7];
				double num9 = lowPrices[barNum - num7];
				for (int i = barNum - num7; i <= barNum; i++)
				{
					if (highPrices[i] > num8)
					{
						num8 = highPrices[i];
					}
					if (lowPrices[i] < num9)
					{
						num9 = lowPrices[i];
					}
				}
				num6 = (pos.get_IsLong() ? num9 : num8);
			}
			num6 = (pos.get_IsLong() ? Math.Max(num6, val) : Math.Min(num6, val));
			double stop = pos.GetStop(barNum);
			if (stop == 0.0)
			{
				return num6;
			}
			if (!pos.get_IsLong())
			{
				return Math.Min(num6, stop);
			}
			return Math.Max(num6, stop);
		}

		// Token: 0x170001E6 RID: 486
		[HandlerParameter(true, "4", Min = "1", Max = "25", Step = "1", Name = "Bars Count")]
		public int BarsCount
		{
			// Token: 0x06000598 RID: 1432 RVA: 0x0001BC40 File Offset: 0x00019E40
			get;
			// Token: 0x06000599 RID: 1433 RVA: 0x0001BC48 File Offset: 0x00019E48
			set;
		}

		// Token: 0x170001E4 RID: 484
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "0.6", Step = "0.1", Name = "Stop Loss")]
		public double StopLoss
		{
			// Token: 0x06000594 RID: 1428 RVA: 0x0001BC1E File Offset: 0x00019E1E
			get;
			// Token: 0x06000595 RID: 1429 RVA: 0x0001BC26 File Offset: 0x00019E26
			set;
		}

		// Token: 0x170001E5 RID: 485
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "0.6", Step = "0.1", Name = "Trail Enable")]
		public double TrailEnable
		{
			// Token: 0x06000596 RID: 1430 RVA: 0x0001BC2F File Offset: 0x00019E2F
			get;
			// Token: 0x06000597 RID: 1431 RVA: 0x0001BC37 File Offset: 0x00019E37
			set;
		}

		// Token: 0x170001E7 RID: 487
		[HandlerParameter(true, "false", NotOptimized = true, Name = "Use Calc Price")]
		public bool UseCalcPrice
		{
			// Token: 0x0600059A RID: 1434 RVA: 0x0001BC51 File Offset: 0x00019E51
			get;
			// Token: 0x0600059B RID: 1435 RVA: 0x0001BC59 File Offset: 0x00019E59
			set;
		}
	}
}
