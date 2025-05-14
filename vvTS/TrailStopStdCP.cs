using System;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000097 RID: 151
	[HandlerCategory("vvPosClose"), HandlerName("TrailStopStd Pct CP")]
	public class TrailStopStdCP : IPosition2Double, IValuesHandler, IHandler, IOneSourceHandler, IDoubleReturns, IPositionInputs
	{
		// Token: 0x06000578 RID: 1400 RVA: 0x0001B6BC File Offset: 0x000198BC
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
				num5 = (num - this.TrailLoss) / 100.0;
				num6 = num2 * (1.0 + (pos.get_IsLong() ? num5 : (-num5)));
			}
			num6 = (pos.get_IsLong() ? Math.Max(num6, val) : Math.Min(num6, val));
			double stop = pos.GetStop(barNum);
			if (stop != 0.0)
			{
				num6 = (pos.get_IsLong() ? Math.Max(num6, stop) : Math.Min(num6, stop));
			}
			if (this.StopStep > 0.0 && this.BarsForStopStep > 0 && TrailStop.IsStopStoned(pos, this.BarsForStopStep, barNum))
			{
				num6 *= 1.0 + (pos.get_IsLong() ? (this.StopStep / 100.0) : (-(this.StopStep / 100.0)));
			}
			return num6;
		}

		// Token: 0x170001D9 RID: 473
		[HandlerParameter(true, "0", Min = "0", Max = "25", Step = "1", Name = "Перенос стопа после(б.)")]
		public int BarsForStopStep
		{
			// Token: 0x06000576 RID: 1398 RVA: 0x0001B6A9 File Offset: 0x000198A9
			get;
			// Token: 0x06000577 RID: 1399 RVA: 0x0001B6B1 File Offset: 0x000198B1
			set;
		}

		// Token: 0x170001D4 RID: 468
		[HandlerParameter(true, "1.5", Min = "0.1", Max = "0.6", Step = "0.05", Name = "Stop Loss")]
		public double StopLoss
		{
			// Token: 0x0600056C RID: 1388 RVA: 0x0001B654 File Offset: 0x00019854
			get;
			// Token: 0x0600056D RID: 1389 RVA: 0x0001B65C File Offset: 0x0001985C
			set;
		}

		// Token: 0x170001D8 RID: 472
		[HandlerParameter(true, "0", Min = "0", Max = "0.5", Step = "0.05", Name = "Шаг переноса стопа")]
		public double StopStep
		{
			// Token: 0x06000574 RID: 1396 RVA: 0x0001B698 File Offset: 0x00019898
			get;
			// Token: 0x06000575 RID: 1397 RVA: 0x0001B6A0 File Offset: 0x000198A0
			set;
		}

		// Token: 0x170001D5 RID: 469
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "0.6", Step = "0.05", Name = "Trail Enable")]
		public double TrailEnable
		{
			// Token: 0x0600056E RID: 1390 RVA: 0x0001B665 File Offset: 0x00019865
			get;
			// Token: 0x0600056F RID: 1391 RVA: 0x0001B66D File Offset: 0x0001986D
			set;
		}

		// Token: 0x170001D6 RID: 470
		[HandlerParameter(true, "0.5", Min = "0.1", Max = "1.5", Step = "0.05", Name = "Trail Loss")]
		public double TrailLoss
		{
			// Token: 0x06000570 RID: 1392 RVA: 0x0001B676 File Offset: 0x00019876
			get;
			// Token: 0x06000571 RID: 1393 RVA: 0x0001B67E File Offset: 0x0001987E
			set;
		}

		// Token: 0x170001D7 RID: 471
		[HandlerParameter(true, "false", NotOptimized = true, Name = "Use Calc Price")]
		public bool UseCalcPrice
		{
			// Token: 0x06000572 RID: 1394 RVA: 0x0001B687 File Offset: 0x00019887
			get;
			// Token: 0x06000573 RID: 1395 RVA: 0x0001B68F File Offset: 0x0001988F
			set;
		}
	}
}
