using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000049 RID: 73
	[HandlerCategory("vvIndicators"), HandlerName("QQE")]
	public class QQE : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600029C RID: 668 RVA: 0x0000C752 File Offset: 0x0000A952
		public IList<double> Execute(IList<double> src)
		{
			return QQE.GenQQE(src, this.Context, this.DrawSignalLine, this.SF, this.RSIperiod, this.Chart);
		}

		// Token: 0x0600029B RID: 667 RVA: 0x0000C398 File Offset: 0x0000A598
		public static IList<double> GenQQE(IList<double> src, IContext ctx, bool DrawSignalLine_, int SF_, int RSIperiod_, bool _Create_QQE_Pane)
		{
			double[] AtrRsi = new double[src.Count];
			double[] array = new double[src.Count];
			int Wilders_Period = RSIperiod_ * 2 - 1;
			IList<double> rsi = ctx.GetData("rsi", new string[]
			{
				RSIperiod_.ToString(),
				src.GetHashCode().ToString()
			}, () => RSI.RSI_TSLab(src, RSIperiod_));
			IList<double> data = ctx.GetData("rsiMa", new string[]
			{
				SF_.ToString(),
				RSIperiod_.ToString(),
				rsi.GetHashCode().ToString()
			}, () => EMA.GenEMA(rsi, SF_));
			for (int i = 1; i < src.Count; i++)
			{
				AtrRsi[i] = Math.Abs(data[i] - data[i - 1]);
			}
			AtrRsi[0] = (AtrRsi[1] = (AtrRsi[2] = AtrRsi[3]));
			IList<double> MaAtrRsi = ctx.GetData("MaAtrRsi", new string[]
			{
				Wilders_Period.ToString(),
				SF_.ToString(),
				RSIperiod_.ToString(),
				AtrRsi.GetHashCode().ToString()
			}, () => EMA.GenEMA(AtrRsi, Wilders_Period));
			IList<double> data2 = ctx.GetData("MaMaAtrRsi", new string[]
			{
				Wilders_Period.ToString(),
				SF_.ToString(),
				RSIperiod_.ToString(),
				MaAtrRsi.GetHashCode().ToString()
			}, () => EMA.GenEMA(MaAtrRsi, Wilders_Period));
			double num = array[src.Count - 1];
			double num2 = data[src.Count - 1];
			for (int j = 0; j < src.Count; j++)
			{
				double num3 = data[j];
				double num4 = data2[j] * 4.236;
				double num5 = num;
				if (num3 < num)
				{
					num = num3 + num4;
					if (num2 < num5 && num > num5)
					{
						num = num5;
					}
				}
				else if (num3 > num)
				{
					num = num3 - num4;
					if (num2 > num5 && num < num5)
					{
						num = num5;
					}
				}
				array[j] = num;
				num2 = num3;
			}
			if (_Create_QQE_Pane)
			{
				IPane pane = ctx.CreatePane("QQE", 35.0, false, false);
				pane.AddList(string.Format(string.Concat(new object[]
				{
					"QQE(RSI:",
					RSIperiod_,
					",SF:",
					SF_.ToString(),
					")"
				}), new object[0]), data, 0, 13042728, 1, 0);
				pane.AddList(string.Format("Trigger", new object[0]), array, 0, 787868, 0, 0);
			}
			if (DrawSignalLine_)
			{
				return array;
			}
			return data;
		}

		// Token: 0x170000DF RID: 223
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Chart
		{
			// Token: 0x06000295 RID: 661 RVA: 0x0000C30F File Offset: 0x0000A50F
			get;
			// Token: 0x06000296 RID: 662 RVA: 0x0000C317 File Offset: 0x0000A517
			set;
		}

		// Token: 0x170000E2 RID: 226
		public IContext Context
		{
			// Token: 0x0600029D RID: 669 RVA: 0x0000C778 File Offset: 0x0000A978
			get;
			// Token: 0x0600029E RID: 670 RVA: 0x0000C780 File Offset: 0x0000A980
			set;
		}

		// Token: 0x170000DE RID: 222
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool DrawSignalLine
		{
			// Token: 0x06000293 RID: 659 RVA: 0x0000C2FE File Offset: 0x0000A4FE
			get;
			// Token: 0x06000294 RID: 660 RVA: 0x0000C306 File Offset: 0x0000A506
			set;
		}

		// Token: 0x170000E1 RID: 225
		[HandlerParameter(true, "14", Min = "10", Max = "30", Step = "1")]
		public int RSIperiod
		{
			// Token: 0x06000299 RID: 665 RVA: 0x0000C331 File Offset: 0x0000A531
			get;
			// Token: 0x0600029A RID: 666 RVA: 0x0000C339 File Offset: 0x0000A539
			set;
		}

		// Token: 0x170000E0 RID: 224
		[HandlerParameter(true, "5", Min = "1", Max = "60", Step = "1")]
		public int SF
		{
			// Token: 0x06000297 RID: 663 RVA: 0x0000C320 File Offset: 0x0000A520
			get;
			// Token: 0x06000298 RID: 664 RVA: 0x0000C328 File Offset: 0x0000A528
			set;
		}
	}
}
