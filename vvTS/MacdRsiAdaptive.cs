using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200014C RID: 332
	[HandlerCategory("vvMACD"), HandlerName("MACD RSI adaptive")]
	public class MacdRsiAdaptive : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000A49 RID: 2633 RVA: 0x0002ACD4 File Offset: 0x00028ED4
		public IList<double> Execute(IList<double> src)
		{
			return MacdRsiAdaptive.GenMacdRsiAdaptive(src, this.Context, this.RSIperiod1, this.Speed1, this.RSIperiod2, this.Speed2, this.SignalPeriod, this.SignalMethod, this.Output);
		}

		// Token: 0x06000A47 RID: 2631 RVA: 0x0002AAF4 File Offset: 0x00028CF4
		public static IList<double> GenMacdRsiAdaptive(IList<double> src, IContext ctx, int _RSIperiod1, double _Speed1, int _RSIperiod2, double _Speed2, int _SignalPeriod, int _SignalMethod, int _Output)
		{
			int count = src.Count;
			double[,] workMaRsi_ = new double[count, 2];
			IList<double> arg_3C_0 = src;
			IList<double> list = new double[count];
			IList<double> data = ctx.GetData("rsi", new string[]
			{
				_RSIperiod1.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.RSI(src, _RSIperiod1));
			IList<double> data2 = ctx.GetData("rsi", new string[]
			{
				_RSIperiod2.ToString(),
				src.GetHashCode().ToString()
			}, () => Series.RSI(src, _RSIperiod2));
			for (int i = 0; i < count; i++)
			{
				list[i] = MacdRsiAdaptive.iMaRsi(src, data, workMaRsi_, _RSIperiod1, _Speed1, i, 0) - MacdRsiAdaptive.iMaRsi(src, data2, workMaRsi_, _RSIperiod2, _Speed2, i, 1);
			}
			IList<double> result = AllAverages.Gen_mMA(list, ctx, _SignalMethod, _SignalPeriod, _SignalPeriod, 1.0, 1.0);
			if (_Output == 1)
			{
				return result;
			}
			return list;
		}

		// Token: 0x06000A48 RID: 2632 RVA: 0x0002AC58 File Offset: 0x00028E58
		private static double iMaRsi(IList<double> src, IList<double> _rsi, double[,] workMaRsi_, int rsiPeriod, double speed, int r, int instanceNo = 0)
		{
			double num = src[r];
			if (r < rsiPeriod)
			{
				workMaRsi_[r, instanceNo] = num;
			}
			else
			{
				workMaRsi_[r, instanceNo] = workMaRsi_[r - 1, instanceNo] + speed * Math.Abs(_rsi[r] / 100.0 - 0.5) * (num - workMaRsi_[r - 1, instanceNo]);
			}
			return workMaRsi_[r, instanceNo];
		}

		// Token: 0x17000362 RID: 866
		public IContext Context
		{
			// Token: 0x06000A4A RID: 2634 RVA: 0x0002AD17 File Offset: 0x00028F17
			get;
			// Token: 0x06000A4B RID: 2635 RVA: 0x0002AD1F File Offset: 0x00028F1F
			set;
		}

		// Token: 0x17000361 RID: 865
		[HandlerParameter(true, "0", NotOptimized = true)]
		public int Output
		{
			// Token: 0x06000A45 RID: 2629 RVA: 0x0002AAB2 File Offset: 0x00028CB2
			get;
			// Token: 0x06000A46 RID: 2630 RVA: 0x0002AABA File Offset: 0x00028CBA
			set;
		}

		// Token: 0x1700035B RID: 859
		[HandlerParameter(true, "14", Min = "1", Max = "30", Step = "1")]
		public int RSIperiod1
		{
			// Token: 0x06000A39 RID: 2617 RVA: 0x0002AA4C File Offset: 0x00028C4C
			get;
			// Token: 0x06000A3A RID: 2618 RVA: 0x0002AA54 File Offset: 0x00028C54
			set;
		}

		// Token: 0x1700035D RID: 861
		[HandlerParameter(true, "34", Min = "1", Max = "30", Step = "1")]
		public int RSIperiod2
		{
			// Token: 0x06000A3D RID: 2621 RVA: 0x0002AA6E File Offset: 0x00028C6E
			get;
			// Token: 0x06000A3E RID: 2622 RVA: 0x0002AA76 File Offset: 0x00028C76
			set;
		}

		// Token: 0x17000360 RID: 864
		[HandlerParameter(true, "0", Min = "0", Max = "1", Step = "1")]
		public int SignalMethod
		{
			// Token: 0x06000A43 RID: 2627 RVA: 0x0002AAA1 File Offset: 0x00028CA1
			get;
			// Token: 0x06000A44 RID: 2628 RVA: 0x0002AAA9 File Offset: 0x00028CA9
			set;
		}

		// Token: 0x1700035F RID: 863
		[HandlerParameter(true, "9", Min = "3", Max = "15", Step = "1")]
		public int SignalPeriod
		{
			// Token: 0x06000A41 RID: 2625 RVA: 0x0002AA90 File Offset: 0x00028C90
			get;
			// Token: 0x06000A42 RID: 2626 RVA: 0x0002AA98 File Offset: 0x00028C98
			set;
		}

		// Token: 0x1700035C RID: 860
		[HandlerParameter(true, "1.2", Min = "0.1", Max = "5", Step = "0.1")]
		public double Speed1
		{
			// Token: 0x06000A3B RID: 2619 RVA: 0x0002AA5D File Offset: 0x00028C5D
			get;
			// Token: 0x06000A3C RID: 2620 RVA: 0x0002AA65 File Offset: 0x00028C65
			set;
		}

		// Token: 0x1700035E RID: 862
		[HandlerParameter(true, "0.8", Min = "0.1", Max = "5", Step = "0.1")]
		public double Speed2
		{
			// Token: 0x06000A3F RID: 2623 RVA: 0x0002AA7F File Offset: 0x00028C7F
			get;
			// Token: 0x06000A40 RID: 2624 RVA: 0x0002AA87 File Offset: 0x00028C87
			set;
		}
	}
}
