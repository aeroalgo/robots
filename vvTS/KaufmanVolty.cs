using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000069 RID: 105
	[HandlerCategory("vvIndicators"), HandlerName("Kaufman Volatility")]
	public class KaufmanVolty : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060003A9 RID: 937 RVA: 0x000145FC File Offset: 0x000127FC
		public IList<double> Execute(IList<double> src)
		{
			int count = src.Count;
			if (this.ERperiod < 1)
			{
				this.ERperiod = 10;
			}
			double[] array = new double[count];
			for (int i = this.ERperiod + 1; i < count; i++)
			{
				double num = this.Volatility(src, i, this.ERperiod);
				if (num != 0.0)
				{
					array[i] = num;
				}
			}
			IList<double> result = array;
			if (this.smooth > 0)
			{
				result = JMA.GenJMA(array, this.smooth, 100);
			}
			return result;
		}

		// Token: 0x060003AA RID: 938 RVA: 0x00014678 File Offset: 0x00012878
		private double Volatility(IList<double> src, int bar, int erperiod)
		{
			if (bar < erperiod + 1)
			{
				return 0.0;
			}
			double num = 0.0;
			for (int i = 0; i < erperiod; i++)
			{
				num += Math.Abs(src[bar - i] - src[bar - 1 - i]);
			}
			return num;
		}

		// Token: 0x1700013B RID: 315
		public IContext Context
		{
			// Token: 0x060003AB RID: 939 RVA: 0x000146C9 File Offset: 0x000128C9
			get;
			// Token: 0x060003AC RID: 940 RVA: 0x000146D1 File Offset: 0x000128D1
			set;
		}

		// Token: 0x17000139 RID: 313
		[HandlerParameter(true, "10", Min = "5", Max = "50", Step = "1")]
		public int ERperiod
		{
			// Token: 0x060003A5 RID: 933 RVA: 0x000145D8 File Offset: 0x000127D8
			get;
			// Token: 0x060003A6 RID: 934 RVA: 0x000145E0 File Offset: 0x000127E0
			set;
		}

		// Token: 0x1700013A RID: 314
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int smooth
		{
			// Token: 0x060003A7 RID: 935 RVA: 0x000145E9 File Offset: 0x000127E9
			get;
			// Token: 0x060003A8 RID: 936 RVA: 0x000145F1 File Offset: 0x000127F1
			set;
		}
	}
}
