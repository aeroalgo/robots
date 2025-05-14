using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200004A RID: 74
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("RAVI (Range Action Verification Index)")]
	public class RAVI : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x060002A5 RID: 677 RVA: 0x0000C938 File Offset: 0x0000AB38
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("RAVI", new string[]
			{
				this.Period1.ToString(),
				this.Period2.ToString(),
				src.GetHashCode().ToString()
			}, () => RAVI.GenRAVI(src, this.Period1, this.Period2, this.Context));
		}

		// Token: 0x060002A4 RID: 676 RVA: 0x0000C7E4 File Offset: 0x0000A9E4
		public static IList<double> GenRAVI(IList<double> _src, int _period1, int _period2, IContext ctx)
		{
			double[] array = new double[_src.Count];
			IList<double> data = ctx.GetData("sma", new string[]
			{
				_period1.ToString(),
				_src.GetHashCode().ToString()
			}, () => SMA.GenSMA(_src, _period1));
			IList<double> data2 = ctx.GetData("sma", new string[]
			{
				_period2.ToString(),
				_src.GetHashCode().ToString()
			}, () => SMA.GenSMA(_src, _period2));
			for (int i = 0; i < _src.Count; i++)
			{
				double num = (data[i] - data2[i]) / data2[i] * 100.0;
				array[i] = num;
			}
			return array;
		}

		// Token: 0x170000E5 RID: 229
		public IContext Context
		{
			// Token: 0x060002A6 RID: 678 RVA: 0x0000C9B6 File Offset: 0x0000ABB6
			get;
			// Token: 0x060002A7 RID: 679 RVA: 0x0000C9BE File Offset: 0x0000ABBE
			set;
		}

		// Token: 0x170000E3 RID: 227
		[HandlerParameter(true, "7", Min = "3", Max = "60", Step = "1")]
		public int Period1
		{
			// Token: 0x060002A0 RID: 672 RVA: 0x0000C791 File Offset: 0x0000A991
			get;
			// Token: 0x060002A1 RID: 673 RVA: 0x0000C799 File Offset: 0x0000A999
			set;
		}

		// Token: 0x170000E4 RID: 228
		[HandlerParameter(true, "65", Min = "20", Max = "120", Step = "1")]
		public int Period2
		{
			// Token: 0x060002A2 RID: 674 RVA: 0x0000C7A2 File Offset: 0x0000A9A2
			get;
			// Token: 0x060002A3 RID: 675 RVA: 0x0000C7AA File Offset: 0x0000A9AA
			set;
		}
	}
}
