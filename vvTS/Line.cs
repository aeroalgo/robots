using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000123 RID: 291
	[HandlerCategory("vvBands&Channels"), HandlerDecimals(0), HandlerName("Line")]
	public class Line : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600085E RID: 2142 RVA: 0x00023643 File Offset: 0x00021843
		public IList<double> Execute(IList<double> _list)
		{
			return Line.GenLine(_list, (double)this.Value);
		}

		// Token: 0x0600085D RID: 2141 RVA: 0x00023610 File Offset: 0x00021810
		public static IList<double> GenLine(IList<double> _list, double _value)
		{
			IList<double> list = new double[_list.Count];
			for (int i = 0; i < _list.Count; i++)
			{
				list[i] = _value;
			}
			return list;
		}

		// Token: 0x170002AA RID: 682
		public IContext Context
		{
			// Token: 0x0600085F RID: 2143 RVA: 0x00023652 File Offset: 0x00021852
			get;
			// Token: 0x06000860 RID: 2144 RVA: 0x0002365A File Offset: 0x0002185A
			set;
		}

		// Token: 0x170002A9 RID: 681
		[HandlerParameter(true, "0", Min = "-1000000", Max = "1000000", Step = "1")]
		public int Value
		{
			// Token: 0x0600085B RID: 2139 RVA: 0x000235FF File Offset: 0x000217FF
			get;
			// Token: 0x0600085C RID: 2140 RVA: 0x00023607 File Offset: 0x00021807
			set;
		}
	}
}
