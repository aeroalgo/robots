using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200013E RID: 318
	[HandlerCategory("vvRSI"), HandlerName("T3 RSI")]
	public class T3RSI : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060009C6 RID: 2502 RVA: 0x00028AE1 File Offset: 0x00026CE1
		public IList<double> Execute(IList<double> src)
		{
			return T3RSI.GenRSI(src, this.Context, this.RSI_Period, this.T3_Period, this.T3_Hot, this.CutlersRSI);
		}

		// Token: 0x060009C5 RID: 2501 RVA: 0x00028810 File Offset: 0x00026A10
		public static IList<double> GenRSI(IList<double> src, IContext ctx, int _rsiperiod, int _T3_Period, double _T3_Hot, bool cutlersrsi)
		{
			int count = src.Count;
			double num = 0.0;
			double num2 = 0.0;
			double num3 = 0.0;
			double num4 = 0.0;
			double num5 = 0.0;
			double num6 = 0.0;
			double[] array = new double[count];
			double num7 = _T3_Hot * _T3_Hot;
			double num8 = num7 * _T3_Hot;
			double num9 = -num8;
			double num10 = 3.0 * (num7 + num8);
			double num11 = -3.0 * (2.0 * num7 + _T3_Hot + num8);
			double num12 = 1.0 + 3.0 * _T3_Hot + num8 + 3.0 * num7;
			double num13 = (double)_T3_Period;
			if (num13 < 1.0)
			{
				num13 = 1.0;
			}
			num13 = 1.0 + 0.5 * (num13 - 1.0);
			double num14 = 2.0 / (num13 + 1.0);
			double num15 = 1.0 - num14;
			IList<double> list = src;
			if (cutlersrsi)
			{
				list = ctx.GetData("cuttlersrsi", new string[]
				{
					_rsiperiod.ToString(),
					src.GetHashCode().ToString()
				}, () => Series.CuttlerRSI(src, _rsiperiod));
			}
			else
			{
				list = ctx.GetData("rsi", new string[]
				{
					_rsiperiod.ToString(),
					src.GetHashCode().ToString()
				}, () => Series.RSI(src, _rsiperiod));
			}
			for (int i = _rsiperiod; i < count; i++)
			{
				num = num14 * list[i] + num15 * num;
				num2 = num14 * num + num15 * num2;
				num3 = num14 * num2 + num15 * num3;
				num4 = num14 * num3 + num15 * num4;
				num5 = num14 * num4 + num15 * num5;
				num6 = num14 * num5 + num15 * num6;
				array[i] = num9 * num6 + num10 * num5 + num11 * num4 + num12 * num3;
			}
			return array;
		}

		// Token: 0x17000333 RID: 819
		public IContext Context
		{
			// Token: 0x060009C7 RID: 2503 RVA: 0x00028B07 File Offset: 0x00026D07
			get;
			// Token: 0x060009C8 RID: 2504 RVA: 0x00028B0F File Offset: 0x00026D0F
			set;
		}

		// Token: 0x17000332 RID: 818
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool CutlersRSI
		{
			// Token: 0x060009C3 RID: 2499 RVA: 0x000287D1 File Offset: 0x000269D1
			get;
			// Token: 0x060009C4 RID: 2500 RVA: 0x000287D9 File Offset: 0x000269D9
			set;
		}

		// Token: 0x1700032F RID: 815
		[HandlerParameter(true, "8", Min = "2", Max = "30", Step = "0")]
		public int RSI_Period
		{
			// Token: 0x060009BD RID: 2493 RVA: 0x0002879E File Offset: 0x0002699E
			get;
			// Token: 0x060009BE RID: 2494 RVA: 0x000287A6 File Offset: 0x000269A6
			set;
		}

		// Token: 0x17000331 RID: 817
		[HandlerParameter(true, "0.618", Min = "0", Max = "1", Step = "0.1")]
		public double T3_Hot
		{
			// Token: 0x060009C1 RID: 2497 RVA: 0x000287C0 File Offset: 0x000269C0
			get;
			// Token: 0x060009C2 RID: 2498 RVA: 0x000287C8 File Offset: 0x000269C8
			set;
		}

		// Token: 0x17000330 RID: 816
		[HandlerParameter(true, "3", Min = "0", Max = "20", Step = "1")]
		public int T3_Period
		{
			// Token: 0x060009BF RID: 2495 RVA: 0x000287AF File Offset: 0x000269AF
			get;
			// Token: 0x060009C0 RID: 2496 RVA: 0x000287B7 File Offset: 0x000269B7
			set;
		}
	}
}
