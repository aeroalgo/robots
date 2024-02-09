using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200014D RID: 333
	[HandlerCategory("vvMACD"), HandlerName("newMACD")]
	public class newMACD : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000A57 RID: 2647 RVA: 0x0002AD88 File Offset: 0x00028F88
		public IList<double> Execute(IList<double> src)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] result = new double[count];
			double[] result2 = new double[count];
			double[] array3 = new double[count];
			double[] result3 = new double[count];
			double num = 2.0 / (this.SignalEMA + 1.0);
			double num2 = 1.0 - num;
			IList<double> list = EMA.GenEMA(src, this.FastEMA);
			IList<double> list2 = EMA.GenEMA(src, this.SlowEMA);
			for (int i = 1; i < count; i++)
			{
				array[i] = list[i] - list2[i];
				array2[i] = num * array[i] + num2 * array2[i - 1];
				array3[i] = array[i] - array2[i];
			}
			if (this.Chart > 0)
			{
				IPane pane = this.Context.CreatePane("-", 40.0, false, false);
				pane.AddList(string.Concat(new string[]
				{
					"newMACD(",
					this.FastEMA.ToString(),
					",",
					this.SlowEMA.ToString(),
					",",
					this.SignalEMA.ToString(),
					")"
				}), array, 1, 8356346, 0, 0);
				pane.AddList("signal(" + this.SignalEMA.ToString() + ")", array2, 0, 13369892, 0, 0);
			}
			if (this.Output == 1)
			{
				return array2;
			}
			if (this.Output == 2)
			{
				return result;
			}
			if (this.Output == 3)
			{
				return result2;
			}
			if (this.Output == 4)
			{
				return result3;
			}
			if (this.Output == 5)
			{
				return array3;
			}
			return array;
		}

		// Token: 0x17000367 RID: 871
		[HandlerParameter(true, "1", Min = "0", Max = "1", Step = "1")]
		public int Chart
		{
			// Token: 0x06000A55 RID: 2645 RVA: 0x0002AD74 File Offset: 0x00028F74
			get;
			// Token: 0x06000A56 RID: 2646 RVA: 0x0002AD7C File Offset: 0x00028F7C
			set;
		}

		// Token: 0x17000368 RID: 872
		public IContext Context
		{
			// Token: 0x06000A58 RID: 2648 RVA: 0x0002AF73 File Offset: 0x00029173
			get;
			// Token: 0x06000A59 RID: 2649 RVA: 0x0002AF7B File Offset: 0x0002917B
			set;
		}

		// Token: 0x17000363 RID: 867
		[HandlerParameter(true, "12", Min = "3", Max = "20", Step = "1")]
		public double FastEMA
		{
			// Token: 0x06000A4D RID: 2637 RVA: 0x0002AD30 File Offset: 0x00028F30
			get;
			// Token: 0x06000A4E RID: 2638 RVA: 0x0002AD38 File Offset: 0x00028F38
			set;
		}

		// Token: 0x17000366 RID: 870
		[HandlerParameter(true, "0", Min = "0", Max = "3", Step = "1")]
		public int Output
		{
			// Token: 0x06000A53 RID: 2643 RVA: 0x0002AD63 File Offset: 0x00028F63
			get;
			// Token: 0x06000A54 RID: 2644 RVA: 0x0002AD6B File Offset: 0x00028F6B
			set;
		}

		// Token: 0x17000365 RID: 869
		[HandlerParameter(true, "9", Min = "3", Max = "20", Step = "1")]
		public double SignalEMA
		{
			// Token: 0x06000A51 RID: 2641 RVA: 0x0002AD52 File Offset: 0x00028F52
			get;
			// Token: 0x06000A52 RID: 2642 RVA: 0x0002AD5A File Offset: 0x00028F5A
			set;
		}

		// Token: 0x17000364 RID: 868
		[HandlerParameter(true, "26", Min = "3", Max = "20", Step = "1")]
		public double SlowEMA
		{
			// Token: 0x06000A4F RID: 2639 RVA: 0x0002AD41 File Offset: 0x00028F41
			get;
			// Token: 0x06000A50 RID: 2640 RVA: 0x0002AD49 File Offset: 0x00028F49
			set;
		}
	}
}
